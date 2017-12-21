use super::types::{Type,ForAll,generalize};
use super::subst::Subst;
use super::env::Env;
use super::unify::unify;
use ::hir::*;
use ::{Result,Error,Vector,fresh_id};
use std::rc::Rc;

fn mk_tycon(str: &str) -> Type {
    Type::TyCon(Rc::new(str.to_string()))
}

fn mk_func(param: Vec<Type>, ret: Type) -> Type {
    use self::Type::*;
    let mk_fn = |ret, param| TyApp(Box::new(mk_tycon("->")), vec![param, ret]);

    let itr = param.into_iter().rev();
    let ty = match itr.len() {
        0 => mk_fn(ret, mk_tycon("unit")),
        _ => itr.fold(ret, mk_fn),
    };
    ty
}



pub (super) fn infer(gamma: &mut Env, expr: &Expr) -> Result<(Subst, Type)> {
    use self::Type::*;
    use self::Expr::*;
    let subst = Subst::new();
    let (subst, ty) = match *expr {
        UnitLit     => (subst, mk_tycon("unit")),
        I32Lit(_)   => (subst, mk_tycon("i32")),
        BoolLit(_)  => (subst, mk_tycon("bool")),
        Var(ref id) => {
            let sigma = gamma.lookup(id)?;
            (subst, sigma.instantiate())
        }
        Lam(ref lam)              => infer_lam(gamma.clone(), lam)?,
        App(ref callee, ref args) => infer_app(gamma, callee, args)?,
        If(ref if_exp)            => infer_if(gamma, if_exp)?,
        Let(ref let_exp)          => infer_let(gamma, let_exp)?,
    };
    Ok((subst, ty))
}

fn infer_lam(mut gamma: Env, lam: &Lam) -> Result<(Subst, Type)> {
    use self::Type::*;
    use self::Expr::*;
    //HACK to handle recursion we should use Y combinator instead
    //let fn_ty = ForAll::new(vec![], TyVar(fresh_id()));
    //gamma.extend(lam.proto().ident(), fn_ty);
    let params_ty = lam.proto().params()
        .iter()
        .map(| id | {
            let tv = fresh_id();
            gamma.extend(id, ForAll::new(vec![], TyVar(tv)));
            TyVar(tv)
        })
        .collect();
    let (s1, t1) = infer(&mut gamma, lam.body())?;
    let fnty     = mk_func(params_ty, t1);
    let fnty     = s1.apply(&fnty);
    Ok((s1, fnty))
}

fn infer_app(gamma: &mut Env, callee: &Box<Expr>, arg: &Box<Expr>)
             -> Result<(Subst, Type)>
{
    let tv = Type::TyVar(fresh_id());
    let (s1, t1)  = infer(gamma, callee)?;
    let mut gamma = gamma.apply_subst(&s1);
    let (s2, t2)  = infer(&mut gamma, arg)?;
    let fnty      = mk_func(vec![t2], tv.clone());
    let s3        = unify(&s2.apply(&t1), &fnty)?;
    let t         = s3.apply(&tv);
    let subst     = s3.compose(&s2)?.
        compose(&s1)?;
    Ok((subst, t))
}

fn infer_let(gamma: &mut Env, let_exp: &Let) -> Result<(Subst, Type)>
{
    let (s1, t1)   = infer(gamma, let_exp.bind())?;
    let mut gamma1 = gamma.apply_subst(&s1);
    let t2         = generalize(t1, &gamma1);
    gamma1.extend(let_exp.id(), t2);
    let (s2, t)    = infer(&mut gamma1, let_exp.expr())?;
    let s          = s2.compose(&s1)?;
    Ok((s, t))
}

fn infer_letrec(gamma: &mut Env, id: &Ident, e: &Expr) -> Result<(Subst, Type)>
{
    /*
    //let (s1, t)   = infer(
    let (s1, t1)   = infer(gamma, let_exp.bind())?;
    let mut gamma1 = gamma.apply_subst(&s1);
    let t2         = generalize(t1, &gamma1);
    gamma1.extend(let_exp.id(), t2);
    let (s2, t)    = infer(&mut gamma1, let_exp.expr())?;
    let s          = s2.compose(&s1)?;
    Ok((s, t))
     */
    unimplemented!();
}

fn infer_if(gamma: &mut Env, if_expr: &If) -> Result<(Subst, Type)>
{
    let (s1, t1) = infer(gamma, if_expr.cond())?;
    let (s2, t2) = infer(gamma, if_expr.texpr())?;
    let (s3, t3) = infer(gamma, if_expr.fexpr())?;

    let s4 = unify(&t1, &mk_tycon("bool"))?;
    let s5 = unify(&t2, &t3)?;

    let ty = s5.apply(&t2);
    let subst = s5.compose(&s4)?.
        compose(&s3)?.
        compose(&s2)?.
        compose(&s1)?;
    Ok((subst, ty))
}
