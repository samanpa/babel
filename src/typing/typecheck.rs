use ::idtree;
use ::xir;
use super::types::{ForAll};
use super::subst::Subst;
use super::hm::{infer_fn,into_xir_symbol};
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
                let v = into_xir_symbol(v, v.ty());
                xir::Decl::Extern(v)
            }
            idtree::Decl::Let(ref bind) => {
                let (s, b)   = infer_fn(&mut self.gamma, bind)?;
                let bind_res = bind_subst(&b, &s);
                /*
                println!("{:?}", bind);
                println!("->\n{:?}", b);
                println!("->\n{:?}", s);
                println!("->\n{:?}\n=================\n\n", bind_res);
                */
                xir::Decl::Let(bind_res)
            }
        };
        Ok(res)
    }
}


fn mk_symbol(tv: &xir::Symbol, sub: &Subst) -> xir::Symbol {
    tv.with_ty(sub.apply(tv.ty()))
}

fn bind_subst(bind: &xir::Bind, sub: &Subst) -> xir::Bind {
    use xir::Bind::*;
    match *bind {
        NonRec{ref symbol, ref expr } => {
            let symbol = mk_symbol(symbol, sub);
            let expr = subst(expr, sub);
            NonRec{symbol, expr}
        }
    }
}

fn subst(expr: &xir::Expr, sub: &Subst) -> xir::Expr
{
    use ::xir::*;
    use xir::Expr::*;
    match *expr {
        UnitLit     => UnitLit,
        I32Lit(n)   => I32Lit(n),
        BoolLit(b)  => BoolLit(b),
        Var(ref id) => Var(mk_symbol(id, sub)),
        Lam(ref proto, ref body, ref retty) => {
            let body  = subst(body, sub);
            let proto = proto.iter()
                .map( |v| mk_symbol(v, sub) )
                .collect();
            Lam(proto, Box::new(body), sub.apply(retty))
        }
        If(ref e) => {
            let if_expr = xir::If::new(subst(e.cond(),  sub),
                                       subst(e.texpr(), sub),
                                       subst(e.fexpr(), sub),
                                       e.ty().clone());
            Expr::If(Box::new(if_expr))
        }
        App(ref callee, ref args) => {
            let callee = subst(callee, sub);
            let args   = args.iter().map(| arg| subst(arg, sub) )
                .collect::<Vec<_>>();
            xir::Expr::App(Box::new(callee), args)
        }
        Let(ref le) => {
            let bind = bind_subst(le.bind(), sub);
            let expr = subst(le.expr(), sub);
            let expr = xir::Let::new(bind, expr);
            Expr::Let(Box::new(expr))
        }
        TyLam(ref args, ref b) => {
            let body  = subst(b, sub);
            TyLam(args.clone(), Box::new(body))
        }
        TyApp(ref e, ref args) => {
            let e = subst(e, sub);
            let args = args.iter()
                .map( |ty| sub.apply(ty) )
                .collect();
            TyApp(Box::new(e), args)
        }
    }
}
    
