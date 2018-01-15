//Standard Hindley Milner augmented with value restriction
//    "Simple imperative polymorphism" - Wright

use super::types::{Type,TyVar,ForAll,fresh_tyvar,generalize};
use super::subst::Subst;
use super::env::Env;
use super::unify::unify;
use ::xir::*;
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
        UnitLit       => (subst, mk_tycon("unit"), UnitLit),
        I32Lit(n)     => (subst, mk_tycon("i32"), I32Lit(n)),
        BoolLit(b)    => (subst, mk_tycon("bool"), BoolLit(b)),
        Var(ref id)   => infer_var(gamma, id)?,
        Lam(ref lam)  => infer_lam(gamma.clone(), lam)?,
        If(ref exp)   => infer_if(gamma, exp)?,
        Let(ref exp)  => infer_let(gamma, exp)?,
        App(ref callee, ref args) => infer_app(gamma, callee, args)?,
        TyLam(_, _)               => unimplemented!(),
        TyApp(_, _)               => unimplemented!(),
    };
    Ok((subst, ty, expr))
}

// Assume the function in the type env (Γ)
//   Γ = { i32_inc : ∀. i32 -> i32,
//         foo     : ∀{a,b}. (a->b) -> a -> b }
// We translate
//       foo inc_i32 1
// as
//   (Λ a1 b1 . (foo {a1, b1})) (Λ . inc_i32 {}) 1
//   read as TyLam([a1,b1],
//              TyApp(Var(foo),
//                     [a1, b1]))
fn translate_var(id: &Ident, tvs: Vec<TyVar>) -> Expr {
    use self::Expr::*;
    let ty_args = tvs.iter()
        .map( |tv| Type::Var(*tv) )
        .collect();
    let tyapp   = TyApp(Box::new(Var(id.clone())), ty_args);
    TyLam(tvs, Box::new(tyapp))
}

fn infer_var(gamma: &mut Env, id: &Ident) -> Result<(Subst, Type, Expr)> {
    let sigma     = gamma.lookup(id)?;
    let (tvs, ty) = sigma.instantiate();
    let expr      = translate_var(id, tvs);
    Ok((Subst::new(), ty, expr))
}

//Adds type abstraction to introduce/close over the free type variables
//   in the body of a lambda. Adds polymorphism to the expression tree
//e.g. the following gets translated as 
//   let foo f x = fx;; aka let foo = λf. λy. f x
//into
//   let foo = Λ a b. ( λf. λy. f x )
//
fn translate_lam(body: Expr, proto: &FnProto, params_ty: &Vec<Type>,
                 retty: &Type, sub: &Subst) -> Expr {
    let params  = proto.params()
        .iter()
        .zip(params_ty)
        .map( |(id,ty)| Ident::new(id.name().clone(), sub.apply(ty), id.id()) )
        .collect::<Vec<_>>();
    let mut free_tv = sub.apply(retty).free_tyvars();
    for p in &params {
        for ftv in p.ty().free_tyvars() {
            free_tv.insert(ftv);
        }
    }
    let free_tv = free_tv.into_iter().collect();
    let proto   = FnProto::new(params);    
    let body    = Expr::TyLam(free_tv, Box::new(body));
    let lam     = Lam::new(proto, body);
    Expr::Lam(Rc::new(lam))
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
    let expr = translate_lam(body, lam.proto(), &params_ty, &t1, &s1);
    let fnty = mk_func(&params_ty, t1);
    let fnty = s1.apply(&fnty);
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

fn is_value(expr: &Expr) -> bool {
    use self::Expr::*;
    match *expr {
        UnitLit    |
        BoolLit(_) |
        I32Lit(_)  |
        Lam(_)     |
        Var(_)  => true,
        _       => false
    }
}

fn infer_let(gamma: &mut Env, let_exp: &Let) -> Result<(Subst, Type, Expr)>
{
    let (s1, t1, e1) = infer(gamma, let_exp.bind())?;
    let id           = let_exp.id();
    let id           = Ident::new(id.name().clone(), t1.clone(), id.id());
    let mut gamma1   = gamma.apply_subst(&s1);
    // Do value restriction: Don't generalize unless the bind expr is a value
    let t2           = match is_value(let_exp.bind()) {
        true  => generalize(t1, &gamma1),
        false => ForAll::new(vec![], t1)
    };
    gamma1.extend(let_exp.id(), t2);
    let (s2, t, e2)  = infer(&mut gamma1, let_exp.expr())?;
    let s            = s2.compose(&s1)?;
    let let_exp      = Let::new(id, e1, e2);
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
