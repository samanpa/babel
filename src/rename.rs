use ast;
use hir;
use {VecUtil,Result,Error};
use scoped_map::ScopedMap;
use std::rc::Rc;
use std::collections::HashMap;


pub struct Rename {
    count: u32,
    names: ScopedMap<String, hir::Var>,
    //Store uniq names across all scopes to reduce memory.
    // FIXME: Is this even worth it?
    uniq_names: HashMap<String, Rc<String>>, 
}

impl ::Pass for Rename {
    type Input  = Vec<ast::TopLevel>; //A list of parsed files
    type Output = Vec<hir::TopLevel>;

    fn run(mut self, toplevel_vec: Self::Input) -> Result<Self::Output> {
        let result = VecUtil::map(&toplevel_vec
                                  , |toplevel| self.rename_toplevel(toplevel));
        Ok(result?)
    }
}

impl Rename {
    pub fn new() -> Self {
        Rename{count: 0,
               names: ScopedMap::new(),
               uniq_names: HashMap::new()
        }
    }

    fn rename_ty(&mut self, ty: &ast::Type) -> hir::Type {
        match *ty {
            ast::Type::Bool => hir::Type::Bool,
            ast::Type::I32  => hir::Type::I32,
            ast::Type::Unit => hir::Type::Unit,
            ast::Type::TyVar(ref v) => {
                let count = self.count;
                self.count = count + 1;
                hir::Type::TyVar(count)
            }
            ast::Type::Function{ ref params_ty, ref return_ty } => {
                let params_ty = params_ty.iter()
                    .map(|ty| self.rename_ty(ty))
                    .collect();
                let return_ty = Box::new(self.rename_ty(return_ty));
                hir::Type::Function{ params_ty, return_ty }
            }
        }
    }

    fn add_var(&mut self, name: String, ty: ast::Type) -> Result<hir::Var> {
        let interned_name = self.uniq_names
            .entry(name.clone())
            .or_insert(Rc::new(name.clone()))
            .clone();
        let ty = self.rename_ty(&ty);
        let var = hir::Var::new(interned_name, ty, self.count);
        if let Some(ref _v) = self.names.insert(name.clone(), var.clone()) {
            return Err(Error::new(format!("Name {} already declared", name)));
        }
        self.count = self.count + 1;
        Ok(var)
    }

    fn rename_toplevel(&mut self, toplevel: &ast::TopLevel)
                       -> Result<hir::TopLevel> {
        let decls = VecUtil::map(toplevel.decls()
                                 , |decl| self.rename_topdecl(decl));
        Ok(hir::TopLevel::new(decls?))
    }

    fn rename_proto(&mut self, proto: &ast::FnProto) -> Result<hir::FnProto> {
        let var = self.add_var(proto.name().clone(), proto.ty())?;
        self.names.begin_scope();
        let params = self.rename_params(proto.params())?;
        self.names.end_scope();
        let return_ty = self.rename_ty(proto.return_ty());
        let proto = hir::FnProto::new(var, params, return_ty);
        Ok(proto)
    }
    
    fn rename_topdecl(&mut self, decl: &ast::TopDecl) -> Result<hir::TopDecl> {
        use ast::TopDecl::*;
        let res = match *decl {
            Extern(ref proto) => hir::TopDecl::Extern(self.rename_proto(proto)?),
            Lam(ref lam)      => hir::TopDecl::Lam(self.rename_lam(lam)?),
            Use{..}           => unimplemented!(),
        };
        Ok(res)
    }

    fn rename_params(&mut self, params: &Vec<ast::Param>) -> Result<Vec<hir::Param>> {
        let mut nparams = Vec::new();
        for param in params {
            let var = self.add_var(param.name().clone(), param.ty().clone())?;
            let return_ty = self.rename_ty(param.ty());
            let param = hir::Param::new(var, return_ty);
            nparams.push(param);
        }
        Ok(nparams)
    }

    fn rename_lam(&mut self, lam: &ast::Lam) -> Result<hir::Lam> {
        let func_ty = lam.ty(); //Fixme
        let func_nm = self.add_var(lam.name().clone(), func_ty)?;
        self.names.begin_scope();
        let params = self.rename_params(lam.params())?;
        let body = self.rename(lam.body())?;
        let return_ty = self.rename_ty(lam.return_ty());
        let lam = hir::Lam::new(func_nm, params, return_ty, body);
        self.names.end_scope();
        Ok(lam)
    }
    
    fn rename(&mut self, expr: &ast::Expr) -> Result<hir::Expr> {
        use ast::Expr::*;
        let res = match *expr {
            UnitLit      => hir::Expr::UnitLit,
            I32Lit(n)    => hir::Expr::I32Lit(n),
            BoolLit(b)   => hir::Expr::BoolLit(b),
            If(ref e)    => {
                let if_expr = hir::If::new(self.rename(e.cond())?,
                                           self.rename(e.texpr())?,
                                           self.rename(e.fexpr())?,
                                           None);
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
            ref expr    => {
                println!("RENAME: NOTHANDLED\n{:?} not handled", expr);
                unimplemented!()
            },
        };
        Ok(res)
    }
}
