use ast;
use hir;
use types::Type;
use {Vector,Result,Error};
use scoped_map::ScopedMap;
use std::rc::Rc;
use std::collections::HashMap;
use fresh_id;

pub struct Rename {
    names: ScopedMap<String, hir::Ident>,
    //Store uniq names across all scopes to reduce memory.
    // FIXME: Is this even worth it?
    uniq_names: HashMap<String, Rc<String>>, 
}

impl ::Pass for Rename {
    type Input  = Vec<ast::Module>; //A list of parsed files
    type Output = Vec<hir::Module>;

    fn run(mut self, mod_vec: Self::Input) -> Result<Self::Output> {
        let result = Vector::map(&mod_vec, |module|
                                  self.rename_module(module));
        println!("{:?}", result);
        Ok(result?)
    }
}

impl Rename {
    pub fn new() -> Self {
        Rename{names: ScopedMap::new(),
               uniq_names: HashMap::new(),
        }
    }
    
    fn rename_ty(&mut self, ty: &ast::Type) -> Result<Type> {
        use ast::Type::*;
        let ty = match *ty {
            TyVar(ref _v)    => Type::TyVar(fresh_id()),
            TyCon(ref tycon) => Type::TyCon(self.mk_tycon(tycon)),
            TyApp(ref con, ref args) => {
                let con  = self.rename_ty(con)?;
                let args = Vector::map(args, |ty| self.rename_ty(ty))?;
                Type::TyApp(Box::new(con), args)
            }
        };
        Ok(ty)
    }

    fn add_uniq_name(&mut self, nm: &String) -> Rc<String>{
        self.uniq_names
            .entry(nm.clone())
            .or_insert(Rc::new(nm.clone()))
            .clone()
    }
        
    fn mk_tycon(&mut self, nm: &String) -> Rc<String> {
        self.add_uniq_name(nm)
    }

    fn insert_ident(&mut self, nm: &String, ident: &hir::Ident) -> Result<()> {
        match self.names.insert(nm.clone(), ident.clone()) {
            None => Ok(()),
            Some(..) => Err(Error::new(format!("Name {} already declared", nm)))
        }
    }

    fn add_ident(&mut self, nm: &String, ty: Type) -> Result<hir::Ident> {
        let ident_name = self.add_uniq_name(nm);
        let ident = hir::Ident::new(ident_name, ty, fresh_id());
        self.insert_ident(nm, &ident)?;
        Ok(ident)
    }
    
    fn rename_module(&mut self, module: &ast::Module) -> Result<hir::Module>
    {
        let decls = Vector::map(module.decls(), |decl| {
            self.rename_decl(decl)
        });
        Ok(hir::Module::new(module.name().clone(), decls?))
    }

    fn rename_extern(&mut self, name: &String, ty: &ast::Type)
                     -> Result<hir::Decl>
    {
        let ty     = self.rename_ty(ty)?;
        let funcid = self.add_ident(name, ty.clone())?;
        
        Ok(hir::Decl::Extern(funcid, ty))
    }
    
    fn new_tyvar() -> Type {
        Type::TyVar(fresh_id())
    }

    fn rename_lam(&mut self, lam: &ast::Lam) ->  Result<hir::Lam>
    {
        self.names.begin_scope();
        let params = Vector::map(lam.params()
                                 , |p| self.add_ident(p, Self::new_tyvar()))?;
        let body   = self.rename(lam.body())?;
        self.names.end_scope();
     
        let proto = hir::FnProto::new(params);
        Ok(hir::Lam::new(proto, body))
    }
    
    fn rename_decl(&mut self, decl: &ast::Decl) -> Result<hir::Decl> {
        use ast::Decl::*;
        let res = match *decl {
            Extern(ref name, ref ty) => {
                self.rename_extern(name, ty)?
            }
            Func(ref name, ref lam) => {
                let fnty = Self::new_tyvar();
                let fnid = self.add_ident(name, fnty)?;
                let lam = self.rename_lam(lam)?;
                hir::Decl::Func(fnid, Rc::new(lam))
            }
        };
        Ok(res)
    }

    fn rename(&mut self, expr: &ast::Expr) -> Result<hir::Expr> {
        use ast::Expr::*;
        let res = match *expr {
            UnitLit      => hir::Expr::UnitLit,
            I32Lit(n)    => hir::Expr::I32Lit(n),
            BoolLit(b)   => hir::Expr::BoolLit(b),
            Lam(ref lam) => {
                let lam = self.rename_lam(lam)?;
                hir::Expr::Lam(Rc::new(lam))
            }
            If(ref e)    => {
                let if_expr = hir::If::new(self.rename(e.cond())?,
                                           self.rename(e.texpr())?,
                                           self.rename(e.fexpr())?);
                hir::Expr::If(Box::new(if_expr))
            }
            App(ref callee, ref arg) => {
                let callee = Box::new(self.rename(callee)?);
                let arg    = Box::new(self.rename(arg)?);
                hir::Expr::App(callee, arg)
            }
            Var(ref nm) => {
                match self.names.get(nm) {
                    Some(v) => hir::Expr::Var(v.clone()),
                    None => {
                        let msg = format!("Could not find variable {}", nm);
                        return Err(Error::new(msg))
                    }
                }
            }
            Let(ref name, ref bind, ref expr) => {
                let letty = Self::new_tyvar();
                let id    = self.add_ident(name, letty)?;
                let bind  = self.rename(bind)?;
                let expr  = self.rename(expr)?;
                let let_  = hir::Let::new(id, bind, expr);
                hir::Expr::Let(Box::new(let_))
            }
        };
        Ok(res)
    }
}
