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

    fn into_xir_tv(var: &idtree::Symbol, ty: &super::types::Type) -> xir::Symbol {
        xir::Symbol::new(var.name().clone(), ty.clone(), var.id())
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
            idtree::Decl::Let(ref bind) => {
                let (s, ty, e) = infer_fn(&mut self.gamma, bind)?;
                let res        = app_subst(&e, &s)?;
                let symbol     = Self::into_xir_tv(bind.symbol(), &ty);
                /*
                println!("{:?}", id);
                println!("->\n{:?}", expr);
                println!("->\n{:?}", e);
                println!("->\n{:?}", s);
                println!("->\n{:?}\n=================\n\n", res);
                */
                xir::Decl::Let(xir::Bind::non_rec(symbol, res))
            }
        };
        Ok(res)
    }
}


fn mk_symbol(tv: &xir::Symbol, sub: &Subst) -> xir::Symbol {
    tv.with_ty(sub.apply(tv.ty()))
}

fn bind_subst(bind: &xir::Bind, sub: &Subst) -> Result<xir::Bind> {
    use xir::Bind::*;
    let bind = match *bind {
        NonRec{ref symbol, ref expr } => {
            let symbol = mk_symbol(symbol, sub);
            let expr = app_subst(expr, sub)?;
            NonRec{symbol, expr}
        }
    };
    Ok(bind)
}
fn app_subst(expr: &xir::Expr, sub: &Subst) -> Result<xir::Expr>
{
    use ::xir::*;
    use xir::Expr::*;
    let expr = match *expr {
        UnitLit     => UnitLit,
        I32Lit(n)   => I32Lit(n),
        BoolLit(b)  => BoolLit(b),
        Var(ref id) => Var(mk_symbol(id, sub)),
        Lam(ref proto, ref body) => {
            let body  = app_subst(body, sub)?;
            let proto = proto.iter()
                .map( |v| mk_symbol(v, sub) )
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
        Let(ref le) => {
            let bind = bind_subst(le.bind(), sub)?;
            let expr = app_subst(le.expr(), sub)?;
            let expr = xir::Let::new(bind, expr);
            Expr::Let(Box::new(expr))
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
    
