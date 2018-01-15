use super::types::{Type,ForAll,fresh_tyvar,generalize};
use super::subst::Subst;
use super::env::Env;
use super::unify::unify;
use ::hir::*;
use ::Result;
use std::rc::Rc;

fn mk_tycon(str: &str) -> Type {
    Type::Con(Rc::new(str.to_string()))
}

fn mk_func(param: &Vec<Type>, ret: Type) -> Type {
    use self::Type::*;
    let mk_fn = |ret, param: &Type| App(Box::new(mk_tycon("->"))
                                          , vec![param.clone(), ret]);

    let itr = param.into_iter().rev();
    match itr.len() {
        0 => mk_fn(ret, &mk_tycon("unit")),
        _ => itr.fold(ret, mk_fn),
    }
}

pub (super) fn infer(gamma: &mut Env, expr: &Expr) -> Result<(Subst, Type, Expr)> {
    use self::Expr::*;
    let subst = Subst::new();
    let (subst, ty, expr) = match *expr {
        UnitLit     => (subst, mk_tycon("unit"), UnitLit),
        I32Lit(n)   => (subst, mk_tycon("i32"), I32Lit(n)),
        BoolLit(b)  => (subst, mk_tycon("bool"), BoolLit(b)),
        Var(ref id) => {
            let sigma = gamma.lookup(id)?;
            (subst, sigma.instantiate(), Var(id.clone()))
        }
        Lam(ref lam)              => infer_lam(gamma.clone(), lam)?,
        App(ref callee, ref args) => infer_app(gamma, callee, args)?,
        If(ref if_exp)            => infer_if(gamma, if_exp)?,
        Let(ref let_exp)          => infer_let(gamma, let_exp)?,
    };
    Ok((subst, ty, expr))
}

fn mk_proto(proto: &FnProto, params_ty: &Vec<Type>, sub: &Subst) -> FnProto {
    let params = proto.params()
        .iter()
        .zip(params_ty)
        .map( |(id,ty)| Ident::new(id.name().clone(), sub.apply(ty), id.id()) )
        .collect();
    FnProto::new(params)
}

fn infer_lam(mut gamma: Env, lam: &Lam) -> Result<(Subst, Type, Expr)> {
    use self::Type::*;
    let params_ty = lam.proto().params()
        .iter()
        .map(| id | {
            let tv = fresh_tyvar();
            gamma.extend(id, ForAll::new(vec![], Var(tv)));
            Var(tv)
        })
        .collect();
    let (s1, t1, body) = infer(&mut gamma, lam.body())?;
    let proto          = mk_proto(lam.proto(), &params_ty, &s1);
    let lam            = Lam::new(proto, body);
    let fnty           = mk_func(&params_ty, t1);
    let fnty           = s1.apply(&fnty);
    let expr           = Expr::Lam(Rc::new(lam));
    Ok((s1, fnty, expr))
}

fn infer_app(gamma: &mut Env, callee: &Expr, arg: &Expr)
             -> Result<(Subst, Type, Expr)>
{
    let (s1, t1, callee) = infer(gamma, callee)?;
    let mut gamma        = gamma.apply_subst(&s1);
    let (s2, t2, arg)    = infer(&mut gamma, arg)?;
    let retty            = Type::Var(fresh_tyvar());
    let fnty             = mk_func(&vec![t2], retty.clone());
    let s3               = unify(&s2.apply(&t1), &fnty)?;
    let t                = s3.apply(&retty);
    let subst            = s3.compose(&s2)?.
        compose(&s1)?;
    let app              = Expr::App(Box::new(callee), Box::new(arg));
    Ok((subst, t, app))
}

fn infer_let(gamma: &mut Env, let_exp: &Let) -> Result<(Subst, Type, Expr)>
{
    let (s1, t1, e1) = infer(gamma, let_exp.bind())?;
    let mut gamma1   = gamma.apply_subst(&s1);
    let t2           = generalize(t1, &gamma1);
    gamma1.extend(let_exp.id(), t2);
    let (s2, t, e2)  = infer(&mut gamma1, let_exp.expr())?;
    let s            = s2.compose(&s1)?;
    let let_exp      = Let::new(let_exp.id().clone(), e1, e2);
    let expr         = Expr::Let(Box::new(let_exp));
    Ok((s, t, expr))
}

pub (super) fn infer_fn(gamma: &mut Env, id: &Ident, e: &Expr) ->
    Result<(Subst, Type, Expr)> {
    infer_letrec(gamma, id, e)
}

fn infer_letrec(gamma: &mut Env, id: &Ident, e: &Expr) 
                -> Result<(Subst, Type, Expr)>
{
    let beta = ForAll::new(vec![], Type::Var(fresh_tyvar()));
    gamma.extend(id, beta);

    let (s1, t1, e) = infer(gamma, e)?;
    Ok((s1, t1, e))
}

fn infer_if(gamma: &mut Env, if_expr: &If) -> Result<(Subst, Type, Expr)>
{
    let (s1, t1, cond) = infer(gamma, if_expr.cond())?;
    let (s2, t2, texp) = infer(gamma, if_expr.texpr())?;
    let (s3, t3, fexp) = infer(gamma, if_expr.fexpr())?;

    let s4 = unify(&t1, &mk_tycon("bool"))?;
    let s5 = unify(&t2, &t3)?;

    let ty = s5.apply(&t2);
    let subst = s5.compose(&s4)?.
        compose(&s3)?.
        compose(&s2)?.
        compose(&s1)?;
    let if_expr = Expr::If(Box::new(If::new(cond, texp, fexp)));
    Ok((subst, ty, if_expr))
}
