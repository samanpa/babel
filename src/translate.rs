// Translates to syntax tree to intermediate representation
//    Called it elaboration in previous incarnation

use ::hir;
use ::ir;
use ::Result;
use std::rc::Rc;

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
            Extern(ref proto, ref tys) => {
                let proto = self.trans_extern(proto, tys)?;
                module.add_extern(proto);
            },
            Lam(ref lam) => {
                let lam = self.trans_lam(lam)?;
                module.add_lambda(lam);
            }
        };
        Ok(())
    }

    fn trans_params(params: &Vec<hir::Ident>) -> Vec<ir::Ident> {
        params.iter()
            .map( |ref param| Self::trans_ident(param) )
            .collect()
    }

    fn trans_ident(ident: &::hir::Ident) -> ir::Ident {
        ir::Ident::new(ident.name().clone(),
                       Self::trans_ty(ident.ty()),
                       ident.id())
    }

    fn trans_ty(ty: &hir::Type) -> ir::Type {
        use hir::Type::*;
        println!("{:?}", ty);
        match *ty {
            Unit => ir::Type::Unit,
            I32  => ir::Type::I32,
            Bool => ir::Type::Bool,
            TyVar(..) => unimplemented!(),
            Function{ ref params_ty, ref return_ty } => {
                let params_ty = params_ty.iter()
                    .map( Self::trans_ty )
                    .collect();
                let return_ty = Box::new(Self::trans_ty(return_ty));
                ir::Type::Function{ params_ty, return_ty }
            }
        }   
    }

    fn trans_extern(&mut self, ext_fun: &hir::Ident
                    , params_ty: &Vec<hir::Type>) -> Result<ir::FnProto>
    {
        let ident = Self::trans_ident(ext_fun);
        let params = params_ty.iter()
            .enumerate()
            .map( |(i, param_ty) | {
                ir::Ident::new( Rc::new(format!("v{}", i))
                                , Self::trans_ty(param_ty)
                                , i as u32)
            }).collect();
        let proto     = ir::FnProto::new(ident, params);
        Ok(proto)
    }
    
    fn trans_lam(&mut self, lam: &hir::Lam) -> Result<ir::Lambda> {
        let params_ty = lam.params().iter()
            .map( |param| Self::trans_ty(param.ty() ) )
            .collect();
        let return_ty = Box::new(Self::trans_ty(lam.ident().ty()));
        let ty = ir::Type::Function{ params_ty, return_ty: return_ty.clone() };
        let ident = ir::Ident::new(lam.ident().name().clone(), ty, lam.ident().id());
        let params = Self::trans_params(lam.params());
        let proto  = ir::FnProto::new(ident, params);
        let body   = self.trans(lam.body())?;
        let lam    = ir::Lambda::new(proto, body);
        Ok(lam)
    }

    fn trans(&mut self, expr: &hir::Expr) -> Result<ir::Expr> {
        use hir::Expr::*;
        let res = match *expr {
            UnitLit    => ir::Expr::UnitLit,
            I32Lit(n)  => ir::Expr::I32Lit(n),
            BoolLit(b) => ir::Expr::BoolLit(b),
            Var(ref v) => ir::Expr::Var(Self::trans_ident(v)),
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
                let ty    = Self::trans_ty(&e.res_ty().clone());
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
