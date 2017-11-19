use ast;
use hir;
use {VecUtil,Result,Error};
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
        let result = VecUtil::map(&toplevel_vec, |toplevel|
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
        let ty = match *ty {
            ast::Type::Bool => hir::Type::Bool,
            ast::Type::I32  => hir::Type::I32,
            ast::Type::Unit => hir::Type::Unit,
            ast::Type::TyVar(ref _v) => hir::Type::TyVar(self.new_count()),
            ast::Type::TyCon(ref tycon) => {
                match self.ty_names.get(tycon) {
                    Some(ref ty) => (*ty).clone(),
                    None => { let msg = format!("TyCon [{}] not found", tycon);
                              return Err(Error::new(msg)) }
                }
            }
            ast::Type::Function{ ref params_ty, ref return_ty } => {
                let params_ty = VecUtil::map(params_ty,
                                             |ty| self.rename_ty(ty))?;
                let return_ty = Box::new(self.rename_ty(return_ty)?);
                hir::Type::Function{ params_ty, return_ty }
            }
        };
        Ok(ty)
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
    
    fn add_tyvar(&mut self, nm: &String) -> Result<u32> {
        let id = self.new_count();
        match self.ty_names.insert(nm.clone(), hir::Type::TyVar(id)) {
            None    => Ok(id),
            Some(_) => Err(Error::new(format!("TyVar {} already declared", nm)))
        }
    }

    fn rename_toplevel(&mut self, toplevel: &ast::TopLevel)
                       -> Result<hir::TopLevel>
    {
        let decls = VecUtil::map(toplevel.decls(), |decl| {
            self.rename_topdecl(decl)
        });
        Ok(hir::TopLevel::new(decls?))
    }

    fn rename_lam(&mut self, proto: &ast::FnProto, body: &ast::Expr)
                  -> Result<hir::Lam> {
        self.ty_names.begin_scope();
        let ty_vars = VecUtil::map(proto.ty_vars(), |ty| self.add_tyvar(ty))?;

        let ty        = self.rename_ty(&proto.ty())?;
        let funcid    = self.add_ident(proto.name(), ty)?;
        let return_ty = self.rename_ty(proto.return_ty())?;        

        self.names.begin_scope();

        let params = VecUtil::map(proto.params(), |param| {
            let ty = self.rename_ty(param.ty())?;
            self.add_ident(param.name(), ty)
        })?;
        let proto   = hir::FnProto::new(funcid, params, ty_vars, return_ty);
        let body    = self.rename(body)?;
        self.names.end_scope();
        self.ty_names.end_scope();
        
        Ok(hir::Lam::new(proto, body))
    }
    
    fn rename_topdecl(&mut self, decl: &ast::TopDecl) -> Result<hir::TopDecl> {
        use ast::TopDecl::*;
        let res = match *decl {
            Extern(ref proto) => {
                let lam = self.rename_lam(proto, &ast::Expr::UnitLit)?;
                hir::TopDecl::Extern(lam.proto1()) //RENAME
            }
            Lam(ref lam) => {
                let lam = self.rename_lam(lam.proto(), lam.body())?;
                hir::TopDecl::Lam(lam)
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
                hir::Expr::Lam(Box::new(lam))
            }
            If(ref e)    => {
                let if_expr = hir::If::new(self.rename(e.cond())?,
                                           self.rename(e.texpr())?,
                                           self.rename(e.fexpr())?,
                                           hir::Type::TyVar(0)); //dummy type
                hir::Expr::If(Box::new(if_expr))
            }
            App{ref callee, ref args} => {
                let callee = Box::new(self.rename(callee)?);
                let args = VecUtil::map(args, |arg| self.rename(arg))?;
                hir::Expr::App{callee, args}                
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
        };
        Ok(res)
    }
}
