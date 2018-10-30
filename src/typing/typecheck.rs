use ::idtree;
use ::xir;
use super::types::{ForAll};
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
                let b = infer_fn(&mut self.gamma, bind, 0)?;
                let bind_res = bind_subst(&b);

                /*
                println!("{:?}", bind);
                println!("->\n{:?}", b);
                println!("->\n{:?}\n=================\n\n", bind_res);
                */
                xir::Decl::Let(bind_res)
            }
        };
        Ok(res)
    }
}


fn mk_symbol(tv: &xir::Symbol) -> xir::Symbol {
    let tv1 = tv.with_ty(tv.ty().apply_subst());
    tv1
}

fn bind_subst(bind: &xir::Bind) -> xir::Bind {
    use xir::Bind::*;
    match *bind {
        NonRec{ref symbol, ref expr } => {
            let symbol = mk_symbol(symbol);
            let expr = subst(expr);
            NonRec{symbol, expr}
        }
    }
}

fn subst(expr: &xir::Expr) -> xir::Expr
{
    use ::xir::*;
    use xir::Expr::*;
    match *expr {
        UnitLit     => UnitLit,
        I32Lit(n)   => I32Lit(n),
        BoolLit(b)  => BoolLit(b),
        Var(ref id) => Var(mk_symbol(id)),
        Lam(ref proto, ref body, ref retty) => {
            let body  = subst(body);
            let proto = proto.iter()
                .map( |v| mk_symbol(v) )
                .collect();
            Lam(proto, Box::new(body), retty.apply_subst())
        }
        If(ref e) => {
            let if_expr = xir::If::new(subst(e.cond()),
                                       subst(e.texpr()),
                                       subst(e.fexpr()),
                                       e.ty().apply_subst());
            Expr::If(Box::new(if_expr))
        }
        App(ref callee, ref args) => {
            let callee = subst(callee);
            let args   = args.iter().map( subst )
                .collect::<Vec<_>>();
            xir::Expr::App(Box::new(callee), args)
        }
        Let(ref le) => {
            let bind = bind_subst(le.bind());
            let expr = subst(le.expr());
            let expr = xir::Let::new(bind, expr);
            Expr::Let(Box::new(expr))
        }
        TyLam(ref args, ref b) => {
            let body  = subst(b);
            TyLam(args.clone(), Box::new(body))
        }
        TyApp(ref e, ref args) => {
            let e = subst(e);
            let args = args.iter()
                .map( |ty| ty.apply_subst() )
                .collect();
            TyApp(Box::new(e), args)
        }
    }
}
    
