// Translates to syntax tree to intermediate representation
//    Called it elaboration in previous incarnation

use ::ast;
use ::ir;
use ::Result;

type VarTy = ::rename::Var;

pub struct Translate {
}

impl ::Pass for Translate {
    type Input = Vec<ast::TopLevel<VarTy>>; //A list of parsed files
    type Output = Vec<ir::Module>;   //A list of modules

    fn run(&mut self, toplevel_vec: Self::Input) -> Result<Self::Output> {
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
                      , toplevel: &ast::TopLevel<VarTy>
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
                     , decl: &ast::TopDecl<VarTy>
                     , module: &mut ir::Module) -> Result<()> {
        use ast::TopDecl::*;
        match *decl {
            Extern(ref proto) => {
                let proto = self.trans_proto(proto);
                module.add_extern(proto);
            },
            Use{..}      => unimplemented!(),
            Lam(ref lam) => {
                let lam = self.trans_lam(&mut &**lam)?;
                module.add_lambda(lam);
            }
        };
        Ok(())
    }

    fn trans_params(&mut self, params: &Vec<ast::Param<::rename::Var>>) -> Vec<ir::Param> {
        params.iter()
            .map( |ref p| ir::Param::new(p.name().name().clone()
                                         , p.name().id() ))
            .collect()
    }

    fn trans_var(var: &::rename::Var) -> ir::Var {
        ir::Var::new(var.name().clone(),
                     Self::trans_ty(var.ty()),
                     var.id())
    }

    fn trans_ty(ty: &ast::Type) -> ir::Type {
        use ast::Type::*;
        use ast::BaseType::*;
        match *ty {
            BaseType(ref b) =>
                match *b {
                    Unit => ir::Type::BaseType(ir::BaseType::Unit),
                    I32  => ir::Type::BaseType(ir::BaseType::I32),
                    Bool => ir::Type::BaseType(ir::BaseType::Bool)
                },
            FunctionType{ ref params_ty, ref return_ty } => {
                let params_ty = params_ty.iter()
                    .map( Self::trans_ty )
                    .collect();
                let return_ty = Box::new(Self::trans_ty(return_ty));
                ir::Type::FunctionType{ params_ty, return_ty }
            }
        }   
    }

    fn trans_proto(&mut self, proto: &ast::FnProto<VarTy>) -> ir::FnProto {
        let name      = Self::trans_var(proto.name());
        let params    = self.trans_params(proto.params());
        let return_ty = Self::trans_ty(proto.return_ty());
        let proto     = ir::FnProto::new(name, params, return_ty);
        proto
    }
    
    fn trans_lam(&mut self, lam: &ast::Lam<VarTy>) -> Result<ir::Lambda> {
        let proto = self.trans_proto(lam.proto());
        let body  = self.trans(lam.body())?;
        let lam   = ir::Lambda::new(proto, body);
        Ok(lam)
    }

    fn trans(&mut self, expr: &ast::Expr<VarTy>) -> Result<ir::Expr> {
        use ast::Expr::*;
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
                ir::Expr::If{ cond:  Box::new(cond),
                              texpr: Box::new(texpr),
                              fexpr: Box::new(fexpr)
                }
            },
            ref expr   => { println!("NOTHANDLED\n{:?} not handled", expr);
                            unimplemented!() },
        };
        Ok(res)
    }
}
