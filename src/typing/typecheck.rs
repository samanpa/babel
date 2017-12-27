use ::{hir,xir};
use ::types::{ForAll};
use super::hm::{infer_fn};
use super::env::Env;
use ::{Result,Vector};

pub struct TypeChecker {
    gamma: Env
}

impl ::Pass for TypeChecker {
    type Input  = Vec<hir::Module>;
    type Output = Vec<xir::Module>;

    fn run(mut self, module_vec: Self::Input) -> Result<Self::Output> {
        let res = Vector::map(&module_vec, |module| self.tc_module(module))?;
        Ok(res)
    }
}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker{ gamma: Env::new() }
    }

    fn tc_module(&mut self, module: &hir::Module) -> Result<xir::Module> {
        let decls = Vector::map(module.decls(), |decl| {
            self.tc_decl(decl)
        })?;
        Ok(xir::Module::new(module.name().clone(), vec![]))
    }

    fn tc_decl(&mut self, decl: &hir::Decl) -> Result<()> {
        use ::hir::Decl::*;
        let res = match *decl {
            Extern(ref id, ref ty) => {
                self.gamma.extend(id, ForAll::new(vec![], ty.clone()));
                //exp_extern(id, ty)?
            }
            Func(ref id, ref lam) => {
                let expr = hir::Expr::Lam(lam.clone());
                let (s, ty) = infer_fn(&mut self.gamma, id, &expr)?;

                let bound_vars = ty.free_tyvars()
                    .iter()
                    .map( |tyvar| *tyvar)
                    .collect();
                self.gamma.extend(id, ForAll::new(bound_vars, ty.clone()));

                println!("{:?} \t===>  {:?}", id.name(), ty);
                //exp_func(id, lam, &s, &ty)?
            }
        };
        Ok(res)
    }
}



