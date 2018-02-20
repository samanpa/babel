use ::xir::*;
use super::types::{Type,ForAll};
use super::subst::Subst;
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
        Ok(Module::new(module.name().clone(), decls))
    }

    fn tc_decl(&mut self, decl: &Decl) -> Result<Decl> {
        use self::Decl::*;
        let res = match *decl {
            Extern(ref v) => {
                self.gamma.extend(v, ForAll::new(vec![], v.ty().clone()));
                Extern(v.clone())
            }
            Let(ref id, ref expr) => {

                let (s, ty, e) = infer_fn(&mut self.gamma, id, expr)?;
                let res        = app_subst(&e, &s)?;
                let bound_vars = ty.free_tyvars()
                    .into_iter()
                    .collect();
                self.gamma.extend(id, ForAll::new(bound_vars, ty.clone()));

                let id = id.with_ty(ty);
                println!("{:?}", id);
                println!("->\n{:?}", expr);
                println!("->\n{:?}", e);
                println!("->\n{:?}", s);
                println!("->\n{:?}\n=================\n\n", res);
                /*
                */
                Let(id, res)
            }
        };
        Ok(res)
    }
}


fn app_subst(expr: &Expr, sub: &Subst) -> Result<Expr>
{
    use ::xir;
    use self::Expr::*;
    let expr = match *expr {
        UnitLit     => UnitLit,
        I32Lit(n)   => I32Lit(n),
        BoolLit(b)  => BoolLit(b),
        Var(ref id) => Var(id.with_ty(sub.apply(id.ty()))),
        Lam(ref proto, ref body) => {
            let body = app_subst(body, sub)?;
            Lam(proto.clone(), Box::new(body))
        }
        If(ref e) => {
            let if_expr = xir::If::new(app_subst(e.cond(),  sub)?,
                                       app_subst(e.texpr(), sub)?,
                                       app_subst(e.fexpr(), sub)?,
                                       e.ty().clone());
            Expr::If(Box::new(if_expr))
        }
        App(n, ref callee, ref arg) => {
            let callee = app_subst(callee, sub)?;
            let arg    = app_subst(arg, sub)?;
            xir::Expr::App(n, Box::new(callee), Box::new(arg))
        }
        Let(ref exp) => {
            let exp = xir::Let::new(exp.id().with_ty(sub.apply(exp.id().ty())),
                                    app_subst(exp.bind(), sub)?,
                                    app_subst(exp.expr(), sub)?);
            Expr::Let(Box::new(exp))
        }
        TyLam(ref args, ref b) => {
            let body  = app_subst(b, sub)?;
            TyLam(args.clone(), Box::new(body))
        }
        TyApp(ref e, ref args) => {
            let e = app_subst(e, sub)?;
            let args = args.iter()
                .map( |ty| {
                    match *ty {
                        Type::Var(tv) => sub.find(tv).map_or(Type::Var(tv),
                                                             | t | t.clone()),
                        _             => ty.clone()
                    }
                })
                .collect();
            TyApp(Box::new(e), args)
        }
    };
    Ok(expr)
}
    
