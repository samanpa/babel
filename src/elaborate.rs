// Translates to syntax tree to intermediate representation

use ::hir;
use ::ir;
use ::Result;

pub struct Elaborate {
    mod_name: String
}

impl ::Pass for Elaborate {
    type Input = Vec<hir::TopLevel>; //A list of parsed files
    type Output = Vec<ir::Module>;   //A list of modules

    fn run(mut self, toplevel_vec: Self::Input) -> Result<Self::Output> {
        let mut mods: Vec<ir::Module> = vec![];
        for toplevel in &toplevel_vec {
            self.run_toplevel(toplevel, &mut mods)?;
        }
        Ok(mods)
    }
}

impl Elaborate {
    pub fn new(mod_name: String) -> Self {
        Elaborate{mod_name}
    }

    fn run_toplevel(&mut self
                      , toplevel: &hir::TopLevel
                      , modules: &mut Vec<ir::Module>) -> Result<()> {
        let mut module = ir::Module::new(self.mod_name.clone());
        for decl in toplevel.decls() {
            self.run_topdecl(decl, &mut module)?
        }
        modules.push(module);
        Ok(())
    }
    
    fn run_topdecl(&mut self
                     , decl: &hir::TopDecl
                     , module: &mut ir::Module) -> Result<()> {
        use hir::TopDecl::*;
        match *decl {
            Extern(ref proto) => {
                let proto = Self::run_proto(proto);
                module.add_extern(proto);
            },
            Lam(ref lam) => {
                let lam = self.run_lam(lam)?;
                module.add_lambda(lam);
            }
        };
        Ok(())
    }

    fn run_params(params: &Vec<hir::Ident>) -> Vec<ir::Ident> {
        params.iter()
            .map( |ref param| Self::run_ident(param) )
            .collect()
    }

    fn run_ident(ident: &::hir::Ident) -> ir::Ident {
        ir::Ident::new(ident.name().clone(),
                       Self::run_ty(ident.ty()),
                       ident.id())
    }

    fn run_ty(ty: &hir::Type) -> ir::Type {
        use hir::Type::*;
        match *ty {
            Unit => ir::Type::Unit,
            I32  => ir::Type::I32,
            Bool => ir::Type::Bool,
            TyVar(_) => unimplemented!(),
            Function{ ref params_ty, ref return_ty } => {
                let params_ty = params_ty.iter()
                    .map( Self::run_ty )
                    .collect();
                let return_ty = Box::new(Self::run_ty(return_ty));
                ir::Type::Function{ params_ty, return_ty }
            }
        }
    }

    fn run_proto(proto: &hir::FnProto) -> ir::FnProto
    {
        let name   = Self::run_ident(proto.ident());
        let params = Self::run_params(proto.params());
        ir::FnProto::new(name, params)
    }
    
    fn run_lam(&mut self, lam: &hir::Lam) -> Result<ir::Lambda> {
        let proto = Self::run_proto(lam.proto());
        let body  = self.trans(lam.body())?;
        let lam   = ir::Lambda::new(proto, body);
        Ok(lam)
    }

    fn trans(&mut self, expr: &hir::Expr) -> Result<ir::Expr> {
        use hir::Expr::*;
        let res = match *expr {
            UnitLit    => ir::Expr::UnitLit,
            I32Lit(n)  => ir::Expr::I32Lit(n),
            BoolLit(b) => ir::Expr::BoolLit(b),
            Var(ref v) => ir::Expr::Var(Self::run_ident(v)),
            App{ref callee, ref args} => {
                let callee = Box::new(self.trans(callee)?);
                let mut args_trans  = Vec::new();
                for arg in args {
                    args_trans.push(self.trans(arg)?);
                }
                ir::Expr::App{callee, args: args_trans}
            }
            If(ref e)  => {
                let cond  = self.trans(e.cond())?;
                let texpr = self.trans(e.texpr())?;
                let fexpr = self.trans(e.fexpr())?;
                let ty    = Self::run_ty(&e.res_ty().clone());
                ir::Expr::If{ cond:  Box::new(cond),
                              texpr: Box::new(texpr),
                              fexpr: Box::new(fexpr),
                              ty
                }
            },
            ref expr   => { println!("NOTHANDLED\n{:?} not handled", expr);
                            unimplemented!() },
        };
        Ok(res)
    }
}
