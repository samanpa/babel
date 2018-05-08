use ast;
use idtree;
use types::{Type,TyCon,fresh_tyvar};
use {Vector,Result,Error};
use scoped_map::ScopedMap;
use std::rc::Rc;
use std::collections::HashMap;
use fresh_id;

pub struct Rename {
    names: ScopedMap<String, idtree::Symbol>,
    //Store uniq names across all scopes to reduce memory.
    // FIXME: Is this even worth it?
    uniq_names: HashMap<String, Rc<String>>, 
}

impl ::Pass for Rename {
    type Input  = Vec<ast::Module>; //A list of parsed files
    type Output = Vec<idtree::Module>;

    fn run(mut self, mod_vec: Self::Input) -> Result<Self::Output> {
        let result = Vector::map(&mod_vec, |module|
                                  self.conv_module(module));
        Ok(result?)
    }
}

impl Default for Rename {
    fn default() -> Self {
        Self::new()
    }
}

impl Rename {
    pub fn new() -> Self {
        Rename{names: ScopedMap::new(),
               uniq_names: HashMap::new(),
        }
    }
    
    fn conv_ty(&mut self, ty: &ast::Type) -> Result<Type> {
        use ast::Type::*;
        let ty = match *ty {
            Var(ref _v)           => Self::new_tyvar(),
            Con(ref nm, ref kind) => {
                let tycon = match nm.as_str() {
                    "i32"  => TyCon::I32,
                    "bool" => TyCon::Bool,
                    "()"   => TyCon::Unit,
                    "->"   => TyCon::Func,
                    _      => TyCon::Cus(self.mk_tycon(nm))
                };
                Type::Con(tycon, kind.clone())
            }
            App(ref con, ref args) => {
                let con = self.conv_ty(con)?;
                let args = Vector::map( args, |arg| self.conv_ty(arg))?;
                Type::App(Box::new(con), args)
            }
        };
        Ok(ty)
    }

    fn add_uniq_name(&mut self, nm: &String) -> Rc<String>{
        self.uniq_names
            .entry(nm.clone())
            .or_insert_with(|| Rc::new(nm.clone()))
            .clone()
    }
        
    fn mk_tycon(&mut self, nm: &String) -> Rc<String> {
        self.add_uniq_name(nm)
    }

    fn insert_var(&mut self, nm: &String, v: &idtree::Symbol) -> Result<()> {
        let at_top_level = self.names.scope() == 0;
        match self.names.insert(nm.clone(), v.clone()) {
            None => Ok(()),
            Some(..) if !at_top_level => Ok(()),
            Some(..) => Err(Error::new(format!("Name {} already declared", nm)))
        }
    }

    fn add_var(&mut self, nm: &String, ty: Type) -> Result<idtree::Symbol> {
        let var_name = self.add_uniq_name(nm);
        let var = idtree::Symbol::new(var_name, ty, fresh_id());
        self.insert_var(nm, &var)?;
        Ok(var)
    }
    
    fn conv_module(&mut self, module: &ast::Module) -> Result<idtree::Module>
    {
        let decls = Vector::map(module.decls(), |decl| {
            self.conv_decl(decl)
        });
        Ok(idtree::Module::new(module.name().clone(), decls?))
    }

    fn conv_extern(&mut self, name: &String, ty: &ast::Type)
                     -> Result<idtree::Decl>
    {
        let ty     = self.conv_ty(ty)?;
        let funcid = self.add_var(name, ty)?;
        
        Ok(idtree::Decl::Extern(funcid))
    }
    
    fn new_tyvar() -> Type {
        Type::Var(fresh_tyvar())
    }

    fn conv_lam(&mut self, lam: &ast::Lam) ->  Result<idtree::Expr>
    {
        self.names.begin_scope();
        let params = Vector::map(lam.params()
                                 , |p| self.add_var(p, Self::new_tyvar()))?;
        let body   = self.conv(lam.body())?;
        self.names.end_scope();
     
        Ok(idtree::Expr::Lam(params, Box::new(body)))
    }

    fn conv_bind(&mut self, bind: &ast::Bind) -> Result<idtree::Bind> {
        match *bind {
            ast::Bind::NonRec(ref name, ref expr) => {
                let lam  = self.conv(expr)?;
                let fnty = Self::new_tyvar();
                let fnid = self.add_var(name, fnty)?;
                Ok(idtree::Bind::new(fnid, lam))
            },
            ast::Bind::Rec(ref name, ref expr) => {
                let fnty = Self::new_tyvar();
                let fnid = self.add_var(name, fnty)?;
                let lam  = self.conv(expr)?;
                Ok(idtree::Bind::new(fnid, lam))
            }
        }
    }
    
    fn conv_decl(&mut self, decl: &ast::Decl) -> Result<idtree::Decl> {
        use ast::Decl::*;
        let res = match *decl {
            Extern(ref name, ref ty) =>
                self.conv_extern(name, ty)?,
            Func(ref bind)           => {
                let bind = self.conv_bind(bind)?;
                idtree::Decl::Let(bind)
            }
        };
        Ok(res)
    }

    fn conv(&mut self, expr: &ast::Expr) -> Result<idtree::Expr> {
        use ast::Expr::*;
        let res = match *expr {
            UnitLit      => idtree::Expr::UnitLit,
            I32Lit(n)    => idtree::Expr::I32Lit(n),
            BoolLit(b)   => idtree::Expr::BoolLit(b),
            Lam(ref lam) => self.conv_lam(lam)?,
            If(ref e)    => {
                let if_expr = idtree::If::new(self.conv(e.cond())?,
                                              self.conv(e.texpr())?,
                                              self.conv(e.fexpr())?,
                                              Self::new_tyvar());
                idtree::Expr::If(Box::new(if_expr))
            }
            App(ref callee, ref args) => {
                let callee = Box::new(self.conv(callee)?);
                let args   = Vector::map(args, |arg| self.conv(arg))?;
                idtree::Expr::App(callee, args)
            }
            Var(ref nm) => {
                match self.names.get(nm) {
                    Some(v) => idtree::Expr::Var(v.clone()),
                    None => {
                        let msg = format!("Could not find variable {}", nm);
                        return Err(Error::new(msg))
                    }
                }
            }
            Let(ref bind, ref expr) => {
                let bind  = self.conv_bind(bind)?;
                let expr  = self.conv(expr)?;
                let let_  = idtree::Let::new(bind, expr);
                idtree::Expr::Let(Box::new(let_))
            }
        };
        Ok(res)
    }
}
