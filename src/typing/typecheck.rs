use ::xir::*;
use super::types::{Type,ForAll};
use super::subst::Subst;
use super::hm::{infer_fn};
use super::env::Env;
use ::{Result,Vector};
use std::rc::Rc;

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
                let expr       = Expr::Lam(lam.clone());
                let (s, ty, e) = infer_fn(&mut self.gamma, id, &expr)?;
                println!("{:?} \t===>  {:?}", id.name(), ty);
                println!("\n{:?} ", e);
                let expr       = insert_tyapp(&e, &s, true)?;

                let bound_vars = ty.free_tyvars()
                    .iter()
                    .cloned()
                    .collect();
                self.gamma.extend(id, ForAll::new(bound_vars, ty.clone()));

                println!("\n{:?} \n\n", expr);
                //exp_func(id, lam, &s, &ty)?
            }
        };
        Ok(res)
    }
}




fn insert_tyapp(expr: &Expr, sub: &Subst, insert: bool) -> Result<Expr>
{
    use ::xir;
    use self::Expr::*;
    let expr = match *expr {
        UnitLit      => UnitLit,
        I32Lit(n)    => I32Lit(n),
        BoolLit(b)   => BoolLit(b),
        Var(ref id)  => Var(id.clone()),
        Lam(ref lam) => {
            let body = insert_tyapp(lam.body(), sub, false)?;
            let lam  = xir::Lam::new(lam.proto().clone(), body);
            Lam(Rc::new(lam))
        }
        If(ref e) => {
            let if_expr = xir::If::new(insert_tyapp(e.cond(),  sub, true)?,
                                       insert_tyapp(e.texpr(), sub, true)?,
                                       insert_tyapp(e.fexpr(), sub, true)?);
            xir::Expr::If(Box::new(if_expr))
        }
        App(ref callee, ref arg) => {
            let callee = insert_tyapp(callee, sub, true)?;
            let arg    = insert_tyapp(arg, sub, true)?;
            xir::Expr::App(Box::new(callee), Box::new(arg))
        }
        Let(ref exp) => {
            let exp = xir::Let::new(exp.id().clone(),
                                    insert_tyapp(exp.bind(), sub, true)?,
                                    insert_tyapp(exp.expr(), sub, true)?);
            xir::Expr::Let(Box::new(exp))
        }
        TyLam(ref args, ref b) => {
            let body  = insert_tyapp(b, sub, true)?;
            let tylam = TyLam(args.clone(), Box::new(body));
            if !insert {
                tylam
            }
            else {
                let args = args.into_iter()
                    .map( |tv| sub.find(*tv).map_or(Type::Var(*tv),
                                                    | t | t.clone()) )
                    .collect();
                TyApp(Box::new(tylam), args)
            }
        }
        TyApp(ref e, ref args) => {
            let e = insert_tyapp(e, sub, true)?;
            TyApp(Box::new(e), args.clone())
        }
        _ => {
            println!("insert_tyapp not implemented {:?}", expr);
            unimplemented!()
        }
    };
    Ok(expr)
}
    
