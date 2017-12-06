use ast;
use hir;
use types::Type;
use {Vector,Result,Error};
use scoped_map::ScopedMap;
use std::rc::Rc;
use std::collections::HashMap;

pub struct Rename {
    count: u32,
    names: ScopedMap<String, hir::Ident>,
    ty_names: ScopedMap<String, hir::Type>,
    //Store uniq names across all scopes to reduce memory.
    // FIXME: Is this even worth it?
    uniq_names: HashMap<String, Rc<String>>, 
}

impl ::Pass for Rename {
    type Input  = Vec<ast::TopLevel>; //A list of parsed files
    type Output = Vec<hir::TopLevel>;

    fn run(mut self, toplevel_vec: Self::Input) -> Result<Self::Output> {
        let result = Vector::map(&toplevel_vec, |toplevel|
                                  self.rename_toplevel(toplevel));
        Ok(result?)
    }
}

impl Rename {
    pub fn new() -> Self {
        Rename{count: 0,
               names: ScopedMap::new(),
               uniq_names: HashMap::new(),
               ty_names: ScopedMap::new(),
        }
    }

    fn new_count(&mut self) -> u32 {
        let count = self.count;
        self.count = count + 1;
        count
    }
    
    fn rename_ty(&mut self, ty: &ast::Type) -> Result<hir::Type> {
        use types::Type::*;
        let ty = match *ty {
            Bool => Bool,
            I32  => I32,
            Unit => Unit,
            TyVar(ref _v) => TyVar(self.new_count()),
            TyCon(ref tycon) => {
                match self.ty_names.get(tycon) {
                    Some(ref ty) => (*ty).clone(),
                    None => { let msg = format!("TyCon [{}] not found", tycon);
                              return Err(Error::new(msg)) }
                }
            }
            Func(ref func_ty) => {
                use types::Function;
                let params_ty = Vector::map(func_ty.params_ty(),
                                             |ty| self.rename_ty(ty))?;
                let return_ty = self.rename_ty(func_ty.return_ty())?;
                let func_ty = Function::new(params_ty, return_ty);
                Func( Box::new(func_ty) )
            }
        };
        Ok(ty)
    }

    fn add_tyvar(&mut self, nm: &String) -> Result<u32> {
        let id = self.new_count();
        match self.ty_names.insert(nm.clone(), Type::TyVar(id)) {
            None    => Ok(id),
            Some(_) => Err(Error::new(format!("TyVar {} already declared", nm)))
        }
    }

    fn rename_ty_scheme(&mut self, forall: &ast::ForAll) -> Result<hir::ForAll>
    {
        //Get numeric identifies for the bound variables and add them as TyVar
        //  to self.ty_names
        let ty_vars = Vector::map(forall.bound_vars(),
                                   |ty| self.add_tyvar(ty))?;
        let ty      = self.rename_ty(forall.ty())?;
        Ok(::types::ForAll::new(ty_vars, ty))
    }
                        
    fn insert_ident(&mut self, nm: &String, ident: &hir::Ident) -> Result<()> {
        match self.names.insert(nm.clone(), ident.clone()) {
            None => Ok(()),
            Some(..) => Err(Error::new(format!("Name {} already declared", nm)))
        }
    }

    fn add_ident(&mut self, nm: &String, ty: hir::Type) -> Result<hir::Ident> {
        let ident_name = self.uniq_names
            .entry(nm.clone())
            .or_insert(Rc::new(nm.clone()))
            .clone();
        let ident = hir::Ident::new(ident_name, ty, self.new_count());
        self.insert_ident(nm, &ident)?;
        Ok(ident)
    }
    
    fn rename_toplevel(&mut self, toplevel: &ast::TopLevel)
                       -> Result<hir::TopLevel>
    {
        let decls = Vector::map(toplevel.decls(), |decl| {
            self.rename_topdecl(decl)
        });
        Ok(hir::TopLevel::new(decls?))
    }

    //body type is &UnitLit when renaming an external declaration
    fn rename_lam(&mut self, proto: &ast::FnProto, body: &ast::Expr)
                  -> Result<hir::Lam>
    {
        self.ty_names.begin_scope();
        let scheme = self.rename_ty_scheme(proto.ty())?;
        let ty     = scheme.ty().clone();
        let funcid = self.add_ident(proto.name(), ty)?;

        self.names.begin_scope();
        let params = Vector::map(proto.params()
                                  , |p| { let ty = self.rename_ty(p.ty())?;
                                          self.add_ident(p.name(), ty) })?;
        let proto  = hir::FnProto::new(funcid, params, scheme);
        let body   = self.rename(body)?;
        self.names.end_scope();
        self.ty_names.end_scope();
        
        Ok(hir::Lam::new(proto, body))
    }
    
    fn rename_topdecl(&mut self, decl: &ast::TopDecl) -> Result<hir::TopDecl> {
        use ast::TopDecl::*;
        let res = match *decl {
            Extern(ref proto) => {
                let lam = self.rename_lam(proto, &ast::Expr::UnitLit)?;
                hir::TopDecl::Extern(lam.proto().clone()) //RENAME
            }
            Lam(ref lam) => {
                let lam = self.rename_lam(lam.proto(), lam.body())?;
                hir::TopDecl::Lam(Rc::new(lam))
            }
            Use{..}           => unimplemented!(),
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
                let lam = self.rename_lam(lam.proto(), lam.body())?;
                hir::Expr::Lam(Rc::new(lam))
            }
            If(ref e)    => {
                let if_expr = hir::If::new(self.rename(e.cond())?,
                                           self.rename(e.texpr())?,
                                           self.rename(e.fexpr())?,
                                           Type::TyVar(0)); //dummy type var
                hir::Expr::If(Box::new(if_expr))
            }
            App{ref callee, ref args} => {
                let callee    = Box::new(self.rename(callee)?);
                let args      = Vector::map(args, |arg| self.rename(arg))?;
                hir::Expr::App{callee, args}
            }
            Var(ref nm, ref ty) => {
                let ty = Vector::map(ty, |ty| self.rename_ty(ty))?;
                match self.names.get(nm) {
                    Some(v) => hir::Expr::Var(v.clone(), ty),
                    None => {
                        let msg = format!("Could not find variable {}", nm);
                        return Err(Error::new(msg))
                    }
                }
            }
        };
        Ok(res)
    }
}
