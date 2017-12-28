use ::hir::*;
use ::types::{ForAll};
use super::hm::{infer_fn};
use super::env::Env;
use ::{Result,Vector};

pub struct TypeChecker {
    gamma: Env
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl ::Pass for TypeChecker {
    type Input  = Vec<Module>;
    type Output = Vec<Module>;

    fn run(mut self, module_vec: Self::Input) -> Result<Self::Output> {
        let res = Vector::map(&module_vec, |module| self.tc_module(module))?;
        Ok(res)
    }
}

impl TypeChecker {
    pub fn new() -> Self {
        TypeChecker{ gamma: Env::new() }
    }

    fn tc_module(&mut self, module: &Module) -> Result<Module> {
        let decls = Vector::map(module.decls(), |decl| {
            self.tc_decl(decl)
        })?;
        Ok(Module::new(module.name().clone(), vec![]))
    }

    fn tc_decl(&mut self, decl: &Decl) -> Result<()> {
        use self::Decl::*;
        let res = match *decl {
            Extern(ref id, ref ty) => {
                self.gamma.extend(id, ForAll::new(vec![], ty.clone()));
                //exp_extern(id, ty)?
            }
            Func(ref id, ref lam) => {
                let expr = Expr::Lam(lam.clone());
                let (_, ty) = infer_fn(&mut self.gamma, id, &expr)?;

                let bound_vars = ty.free_tyvars()
                    .iter()
                    .cloned()
                    .collect();
                self.gamma.extend(id, ForAll::new(bound_vars, ty.clone()));

                println!("{:?} \t===>  {:?}", id.name(), ty);
                //exp_func(id, lam, &s, &ty)?
            }
        };
        Ok(res)
    }
}



