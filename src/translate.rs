// Translates to syntax tree to intermediate representation
//    Called it elaboration in previous incarnation

use ::hir;
use ::ir;
use ::Result;

pub struct Translate {
}

impl ::Pass for Translate {
    type Input = Vec<hir::TopLevel>; //A list of parsed files
    type Output = Vec<ir::Module>;   //A list of modules

    fn run(mut self, toplevel_vec: Self::Input) -> Result<Self::Output> {
        let mut mods: Vec<ir::Module> = vec![];
        for toplevel in &toplevel_vec {
            self.trans_toplevel(toplevel, &mut mods)?;
        }
        Ok(mods)
    }
}

impl Translate {
    pub fn new() -> Self {
        Translate{}
    }

    fn trans_toplevel(&mut self
                      , toplevel: &hir::TopLevel
                      , modules: &mut Vec<ir::Module>) -> Result<()> {
        //FIXME: find better way
        let mut module = ir::Module::new("main".to_string());
        for decl in toplevel.decls() {
            self.trans_topdecl(decl, &mut module)?
        }
        modules.push(module);
        Ok(())
    }
    
    fn trans_topdecl(&mut self
                     , decl: &hir::TopDecl
                     , module: &mut ir::Module) -> Result<()> {
        use hir::TopDecl::*;
        match *decl {
            Extern(ref proto) => {
                let proto = self.trans_proto(proto);
                module.add_extern(proto);
            },
            Lam(ref lam) => {
                let lam = self.trans_lam(lam)?;
                module.add_lambda(lam);
            }
        };
        Ok(())
    }

    fn trans_params(&mut self, params: &Vec<hir::Param>) -> Vec<ir::Param> {
        params.iter()
            .map( |ref p| ir::Param::new(p.name().name().clone()
                                         , p.name().id() ))
            .collect()
    }

    fn trans_var(var: &::hir::Var) -> ir::Var {
        ir::Var::new(var.name().clone(),
                     Self::trans_ty(var.ty()),
                     var.id())
    }

    fn trans_ty(ty: &hir::Type) -> ir::Type {
        use hir::Type::*;
        match *ty {
            Unit => ir::Type::Unit,
            I32  => ir::Type::I32,
            Bool => ir::Type::Bool,
            Function{ ref params_ty, ref return_ty } => {
                let params_ty = params_ty.iter()
                    .map( Self::trans_ty )
                    .collect();
                let return_ty = Box::new(Self::trans_ty(return_ty));
                ir::Type::Function{ params_ty, return_ty }
            }
        }   
    }

    fn trans_proto(&mut self, proto: &hir::FnProto) -> ir::FnProto {
        let name      = Self::trans_var(proto.name());
        let params    = self.trans_params(proto.params());
        let return_ty = Self::trans_ty(proto.return_ty());
        let proto     = ir::FnProto::new(name, params, return_ty);
        proto
    }
    
    fn trans_lam(&mut self, lam: &hir::Lam) -> Result<ir::Lambda> {
        let proto = self.trans_proto(lam.proto());
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
            Var(ref v) => ir::Expr::Var(Self::trans_var(v)),
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
                let ty    = Self::trans_ty(&e.res_ty().clone().unwrap());
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
