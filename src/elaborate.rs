// Translates to syntax tree to intermediate representation

use hir;
use ir;
use Result;
use Vector;

pub struct Elaborate {
    mod_name: String,
    mods: Vec<ir::Module>,
}

impl ::Pass for Elaborate {
    type Input  = Vec<hir::TopLevel>; //A list of parsed files
    type Output = Vec<ir::Module>;    //A list of modules

    fn run(mut self, toplevel_vec: Self::Input) -> Result<Self::Output> {
        for toplevel in toplevel_vec {
            self.elab_toplevel(toplevel)?;
        }
        Ok(self.mods)
    }
}

impl Elaborate {
    pub fn new(mod_name: String) -> Self {
        Elaborate{mod_name, mods: vec![]}
    }

    fn elab_toplevel(&mut self, toplevel: hir::TopLevel) -> Result<()> {
        let mut module = ir::Module::new(self.mod_name.clone());
        for decl in toplevel.decls() {
            self.elab_topdecl(decl, &mut module)?;
        }
        self.mods.push(module);
        Ok(())
    }

    fn elab_topdecl(&mut self, decl: hir::TopDecl
                    , module: &mut ir::Module) -> Result<()> {
        use hir::TopDecl::*;
        match decl {
            Extern(proto) => {
                let proto = Self::elab_proto(&proto);
                module.add_extern(proto);
            },
            Lam(lam) => {
                let lam = self.elab_lam(&lam)?;
                module.add_lambda(lam);
            }
        };
        Ok(())
    }

    fn elab_params(params: &Vec<hir::Ident>) -> Vec<ir::Ident> {
        params.iter()
            .map( |ref param| Self::elab_ident(param) )
            .collect()
    }

    fn elab_ident(ident: &::hir::Ident) -> ir::Ident {
        ir::Ident::new(ident.name().clone(),
                       Self::elab_ty(ident.ty()),
                       ident.id())
    }

    fn elab_ty(ty: &hir::Type) -> ir::Type {
        use types::Type::*;
        match *ty {
            Unit => ir::Type::Unit,
            I32  => ir::Type::I32,
            Bool => ir::Type::Bool,
            TyVar(_, _) => unimplemented!(),
            TyCon(_) => unimplemented!(),
            Func(ref fn_ty) => {
                let params_ty = fn_ty.params_ty().iter()
                    .map( Self::elab_ty )
                    .collect();
                let return_ty = Box::new(Self::elab_ty(fn_ty.return_ty()));
                ir::Type::Function{ params_ty, return_ty }
            }
        }
    }

    fn elab_proto(proto: &hir::FnProto) -> ir::FnProto
    {
        let name   = Self::elab_ident(proto.ident());
        let params = Self::elab_params(proto.params());
        ir::FnProto::new(name, params)
    }
    
    fn elab_lam(&mut self, lam: &hir::Lam) -> Result<ir::Lambda> {
        let proto = Self::elab_proto(lam.proto());
        let body  = self.trans(lam.body())?;
        let lam   = ir::Lambda::new(proto, body);
        Ok(lam)
    }

    fn trans(&mut self, expr: &hir::Expr) -> Result<ir::Expr> {
        use hir::Expr::*;
        //println!("{:?}", expr);
        let res = match *expr {
            UnitLit       => ir::Expr::UnitLit,
            I32Lit(n)     => ir::Expr::I32Lit(n),
            BoolLit(b)    => ir::Expr::BoolLit(b),
            Var(ref v, _) => ir::Expr::Var(Self::elab_ident(v)),
            App{ref callee, ref args} => {
                let callee = Box::new(self.trans(callee)?);
                let args = Vector::map(args, |arg| self.trans(arg))?;
                ir::Expr::App{callee, args}
            }
            If(ref e)  => {
                let cond  = self.trans(e.cond())?;
                let texpr = self.trans(e.texpr())?;
                let fexpr = self.trans(e.fexpr())?;
                let ty    = Self::elab_ty(&e.res_ty().clone());
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
