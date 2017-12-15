use super::types::{Type,Function,ForAll};
use super::subst::Subst;
use super::env::Env;
use super::unify::unify;
use ::hir::*;
use ::{Result,Error,Vector,fresh_id};

pub (super) fn infer(gamma: &Env, expr: &Expr) -> Result<(Subst, Type<u32>)> {
    use self::Type::*;
    use self::Expr::*;
    let subst = Subst::new();
    let (subst, ty) = match *expr {
        UnitLit        => (subst, Unit),
        I32Lit(_)      => (subst, I32),
        BoolLit(_)     => (subst, Bool),
        Var(ref id, _) => {
            let sigma = gamma.lookup(id)?;
            (subst, sigma.instantiate())
        }
        Lam(ref lam)              => infer_lam(gamma.clone(), lam)?,
        App(ref callee, ref args) => infer_app(gamma, callee, args)?,
        If(ref if_exp)            => infer_if(gamma, if_exp)?,
    };
    Ok((subst, ty))
}

fn infer_lam(mut gamma: Env, lam: &Lam) -> Result<(Subst, Type<u32>)> {
    use self::Type::*;
    use self::Expr::*;
    //HACK to handle recursion we should use Y combinator instead
    let fn_ty = ForAll::new(vec![], TyVar(fresh_id()));
    gamma.extend(lam.proto().ident(), fn_ty);
    let params_ty = lam.proto().params()
        .iter()
        .map(| id | {
            let tv = fresh_id();
            gamma.extend(id, ForAll::new(vec![], TyVar(tv)));
            TyVar(tv)
        })
        .collect();
    let (s1, t1) = infer(&gamma, lam.body())?;
    let fn_ty    = Function::new(params_ty, t1.clone());
    let fn_ty    = s1.apply(&Func(Box::new(fn_ty)));
    Ok((s1, fn_ty))
}

fn infer_app(gamma: &Env, callee: &Box<Expr>, args: &Vec<Expr>)
             -> Result<(Subst, Type<u32>)>
{
    unimplemented!()
}

fn infer_if(gamma: &Env, if_expr: &If) -> Result<(Subst, Type<u32>)>
{
    let (s1, t1) = infer(gamma, if_expr.cond())?;
    let (s2, t2) = infer(gamma, if_expr.texpr())?;
    let (s3, t3) = infer(gamma, if_expr.fexpr())?;

    let s4 = unify(&t1, &Type::Bool)?;
    let s5 = unify(&t2, &t3)?;

    let ty = s5.apply(&t2);
    let subst = s5.compose(&s4).compose(&s3).compose(&s2).compose(&s1);
    Ok((subst, ty))
}
