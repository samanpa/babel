use ast;
use hir;
use {VecUtil,Result,Error};
use scoped_map::ScopedMap;
use std::rc::Rc;
use std::collections::HashMap;


pub struct Rename {
    count: u32,
    names: ScopedMap<String, hir::Ident>,
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
               uniq_names: HashMap::new()
        }
    }

    fn new_count(&mut self) -> u32 {
        let count = self.count;
        self.count = count + 1;
        count
    }
    
    fn rename_ty(&mut self, ty: &ast::Type) -> hir::Type {
        match *ty {
            ast::Type::Bool => hir::Type::Bool,
            ast::Type::I32  => hir::Type::I32,
            ast::Type::Unit => hir::Type::Unit,
            ast::Type::TyVar(ref v) => hir::Type::TyVar(self.new_count()),
            ast::Type::Function{ ref params_ty, ref return_ty } => {
                let params_ty = params_ty.iter()
                    .map(|ty| self.rename_ty(ty))
                    .collect();
                let return_ty = Box::new(self.rename_ty(return_ty));
                hir::Type::Function{ params_ty, return_ty }
            }
        }
    }

    fn insert_ident(&mut self, nm: &String, ident: &hir::Ident) -> Result<()> {
        if let Some(..) = self.names.insert(nm.clone(), ident.clone()) {
            return Err(Error::new(format!("Name {} already declared", nm)));
        }
        Ok(())
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
        let decls = VecUtil::map(toplevel.decls(), |decl| {
            self.rename_topdecl(decl)
        });
        Ok(hir::TopLevel::new(decls?))
    }

    fn rename_params(&mut self, params: &Vec<ast::Param>) 
                     -> Result<Vec<hir::Ident>>
    {
        VecUtil::map(params, |param| {
            let ty = self.rename_ty(param.ty());
            self.add_ident(param.name(), ty)
        })
    }

    fn rename_proto(&mut self, proto: &ast::FnProto)-> Result<hir::FnProto> {
        let ty        = self.rename_ty(&proto.ty());
        let funcid    = self.add_ident(proto.name(), ty)?;
        let return_ty = self.rename_ty(proto.return_ty());
        self.names.begin_scope();
        let params    = self.rename_params(proto.params())?;
        self.names.end_scope();
        Ok(hir::FnProto::new(funcid, params, return_ty))
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

    fn rename_lam(&mut self, lam: &ast::Lam) -> Result<hir::Lam> {
        let proto = self.rename_proto(lam.proto())?;
        self.names.begin_scope();
        //Insert function parameters into current scope
        VecUtil::map(proto.params(), |p| self.insert_ident(p.name(), p))?;
        let body  = self.rename(lam.body())?;
        self.names.end_scope();
        let lam   = hir::Lam::new(proto, body);
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
            ref expr    => {
                println!("RENAME: NOTHANDLED\n{:?} not handled", expr);
                unimplemented!()
            },
        };
        Ok(res)
    }
}
