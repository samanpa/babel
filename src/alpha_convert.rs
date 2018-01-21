use ast;
use xir;
use types::{Type,fresh_tyvar};
use {Vector,Result,Error};
use scoped_map::ScopedMap;
use std::rc::Rc;
use std::collections::HashMap;
use fresh_id;

pub struct AlphaConversion {
    names: ScopedMap<String, xir::TermVar>,
    //Store uniq names across all scopes to reduce memory.
    // FIXME: Is this even worth it?
    uniq_names: HashMap<String, Rc<String>>, 
}

impl ::Pass for AlphaConversion {
    type Input  = Vec<ast::Module>; //A list of parsed files
    type Output = Vec<xir::Module>;

    fn run(mut self, mod_vec: Self::Input) -> Result<Self::Output> {
        let result = Vector::map(&mod_vec, |module|
                                  self.conv_module(module));
        Ok(result?)
    }
}

impl Default for AlphaConversion {
    fn default() -> Self {
        Self::new()
    }
}

impl AlphaConversion {
    pub fn new() -> Self {
        AlphaConversion{names: ScopedMap::new(),
                        uniq_names: HashMap::new(),
        }
    }
    
    fn conv_ty(&mut self, ty: &ast::Type) -> Result<Type> {
        use ast::Type::*;
        let ty = match *ty {
            Var(ref _v)    => Self::new_tyvar(),
            Con(ref tycon) => Type::Con(self.mk_tycon(tycon)),
            App(ref con, ref args) => {
                let con  = self.conv_ty(con)?;
                let args = Vector::map(args, |ty| self.conv_ty(ty))?;
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

    fn insert_var(&mut self, nm: &String, v: &xir::TermVar) -> Result<()> {
        match self.names.insert(nm.clone(), v.clone()) {
            None => Ok(()),
            Some(..) => Err(Error::new(format!("Name {} already declared", nm)))
        }
    }

    fn add_var(&mut self, nm: &String, ty: Type) -> Result<xir::TermVar> {
        let var_name = self.add_uniq_name(nm);
        let var = xir::TermVar::new(var_name, ty, fresh_id());
        self.insert_var(nm, &var)?;
        Ok(var)
    }
    
    fn conv_module(&mut self, module: &ast::Module) -> Result<xir::Module>
    {
        let decls = Vector::map(module.decls(), |decl| {
            self.conv_decl(decl)
        });
        Ok(xir::Module::new(module.name().clone(), decls?))
    }

    fn conv_extern(&mut self, name: &String, ty: &ast::Type)
                     -> Result<xir::Decl>
    {
        let ty     = self.conv_ty(ty)?;
        let funcid = self.add_var(name, ty.clone())?;
        
        Ok(xir::Decl::Extern(funcid, ty))
    }
    
    fn new_tyvar() -> Type {
        Type::Var(fresh_tyvar())
    }

    fn conv_lam(&mut self, lam: &ast::Lam) ->  Result<xir::Expr>
    {
        self.names.begin_scope();
        let params = Vector::map(lam.params()
                                 , |p| self.add_var(p, Self::new_tyvar()))?;
        let body   = self.conv(lam.body())?;
        self.names.end_scope();
     
        Ok(xir::Expr::Lam(params, Box::new(body)))
    }
    
    fn conv_decl(&mut self, decl: &ast::Decl) -> Result<xir::Decl> {
        use ast::Decl::*;
        let res = match *decl {
            Extern(ref name, ref ty) => {
                self.conv_extern(name, ty)?
            }
            Func(ref name, ref lam) => {
                let fnty = Self::new_tyvar();
                let fnid = self.add_var(name, fnty)?;
                let lam = self.conv_lam(lam)?;
                xir::Decl::Let(fnid, lam)
            }
        };
        Ok(res)
    }

    fn conv(&mut self, expr: &ast::Expr) -> Result<xir::Expr> {
        use ast::Expr::*;
        let res = match *expr {
            UnitLit      => xir::Expr::UnitLit,
            I32Lit(n)    => xir::Expr::I32Lit(n),
            BoolLit(b)   => xir::Expr::BoolLit(b),
            Lam(ref lam) => self.conv_lam(lam)?,
            If(ref e)    => {
                let if_expr = xir::If::new(self.conv(e.cond())?,
                                           self.conv(e.texpr())?,
                                           self.conv(e.fexpr())?);
                xir::Expr::If(Box::new(if_expr))
            }
            App(ref callee, ref arg) => {
                let callee = Box::new(self.conv(callee)?);
                let arg    = Box::new(self.conv(arg)?);
                xir::Expr::App(callee, arg)
            }
            Var(ref nm) => {
                match self.names.get(nm) {
                    Some(v) => xir::Expr::Var(v.clone()),
                    None => {
                        let msg = format!("Could not find variable {}", nm);
                        return Err(Error::new(msg))
                    }
                }
            }
            Let(ref name, ref bind, ref expr) => {
                let letty = Self::new_tyvar();
                let bind  = self.conv(bind)?;
                let id    = self.add_var(name, letty)?;
                let expr  = self.conv(expr)?;
                let let_  = xir::Let::new(id, bind, expr);
                xir::Expr::Let(Box::new(let_))
            }
        };
        Ok(res)
    }
}
