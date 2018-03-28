use ::idtree;
use ::xir;
use super::types::{ForAll};
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
    type Input  = Vec<idtree::Module>;
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

    fn into_xir_tv(var: &idtree::TermVar, ty: &super::types::Type) -> xir::TermVar {
        xir::TermVar::new(var.name().clone(), ty.clone(), var.id())
    }

    fn tc_module(&mut self, module: &idtree::Module) -> Result<xir::Module> {
        let decls = Vector::map(module.decls(), |decl| {
            self.tc_decl(decl)
        })?;
        Ok(xir::Module::new(module.name().clone(), decls))
    }

    fn tc_decl(&mut self, decl: &idtree::Decl) -> Result<xir::Decl> {
        let res = match *decl {
            idtree::Decl::Extern(ref v) => {
                self.gamma.extend(v, ForAll::new(vec![], v.ty().clone()));
                let v = Self::into_xir_tv(v, v.ty());
                xir::Decl::Extern(v)
            }
            idtree::Decl::Let(ref id, ref expr) => {

                let (s, ty, e) = infer_fn(&mut self.gamma, id, expr)?;
                let res        = app_subst(&e, &s)?;
                let id         = Self::into_xir_tv(id, &ty);
                /*
                println!("{:?}", id);
                println!("->\n{:?}", expr);
                println!("->\n{:?}", e);
                println!("->\n{:?}", s);
                println!("->\n{:?}\n=================\n\n", res);
                */
                xir::Decl::Let(id, res)
            }
        };
        Ok(res)
    }
}


fn app_subst(expr: &xir::Expr, sub: &Subst) -> Result<xir::Expr>
{
    use ::xir::*;
    use xir::Expr::*;
    let expr = match *expr {
        UnitLit     => UnitLit,
        I32Lit(n)   => I32Lit(n),
        BoolLit(b)  => BoolLit(b),
        Var(ref id) => Var(id.with_ty(sub.apply(id.ty()))),
        Lam(ref proto, ref body) => {
            let body  = app_subst(body, sub)?;
            let proto = proto.iter()
                .map( |v| v.with_ty(sub.apply(v.ty())) )
                .collect();
            Lam(proto, Box::new(body))
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
                .map( |ty| sub.apply(ty) )
                .collect();
            TyApp(Box::new(e), args)
        }
    };
    Ok(expr)
}
    
