//Standard Hindley Milner augmented with value restriction
//    "Simple imperative polymorphism" - Wright

use super::types::{Type,TyVar,ForAll,fresh_tyvar,generalize};
use super::subst::Subst;
use super::env::Env;
use super::unify::unify;
use ::xir::*;
use ::Result;
use std::rc::Rc;

fn mk_tycon(str: &str, arity: u32) -> Type {
    Type::Con(Rc::new(str.to_string()), arity)
}

fn mk_fn(param: Type, ret: Type, arity: u32) -> Type {
    use self::Type::*;
    let fncon = Box::new(mk_tycon("->", arity));
    App(Box::new(App(fncon, Box::new(param))), Box::new(ret))
}

fn mk_func(param: &Vec<Type>, ret: Type) -> Type {
    let itr = param.into_iter().rev();
    match itr.len() as u32 {
        0 => mk_fn(mk_tycon("unit", 0), ret, 2),
        _ => {
            let mut ty = ret;
            let mut arity = 2;
            for param in itr {
                ty = mk_fn(param.clone(), ty, arity);
                arity += 1;
            };
            ty
        },
    }
}

pub (super) fn infer(gamma: &mut Env, expr: &Expr) -> Result<(Subst, Type, Expr)> {
    use self::Expr::*;
    let subst = Subst::new();
    let (subst, ty, expr) = match *expr {
        UnitLit       => (subst, mk_tycon("unit", 0), UnitLit),
        I32Lit(n)     => (subst, mk_tycon("i32", 0), I32Lit(n)),
        BoolLit(b)    => (subst, mk_tycon("bool", 0), BoolLit(b)),
        Var(ref v)    => infer_var(gamma, v)?,
        If(ref exp)   => infer_if(gamma, exp)?,
        Let(ref exp)  => infer_let(gamma, exp)?,
        App(n, ref callee, ref args) => infer_app(n, gamma, callee, args)?,
        Lam(ref proto, ref body)     => infer_lam(gamma.clone(), proto, body)?,
        TyLam(_, _)                  => unimplemented!(),
        TyApp(_, _)                  => unimplemented!(),
    };
    Ok((subst, ty, expr))
}

// Assume the type env (Γ)
//   Γ = { i32_inc : ∀. i32 -> i32,
//         foo     : ∀{a,b}. (a->b) -> a -> b }
// We translate
//       foo inc_i32 1
// as
//   (foo {a1, b1}) inc_i32 1
//   read as TyApp(Var(foo),
//                 [a1, b1])
fn translate_var(sigma: &ForAll, var: &TermVar, tvs: Vec<TyVar>) -> Expr {
    use self::Expr::*;
    let ty_args = tvs.iter()
        .map( |tv| Type::Var(*tv) )
        .collect::<Vec<_>>();
    let var     = var.with_ty(sigma.ty().clone());
    let var     = Expr::Var(var);
    match ty_args.len() {
        0 => var,
        _ => TyApp(Box::new(var), ty_args)
    }
}

fn infer_var(gamma: &mut Env, var: &TermVar) -> Result<(Subst, Type, Expr)> {
    let sigma     = gamma.lookup(var)?;
    let (tvs, ty) = sigma.instantiate();
    let expr      = translate_var(&sigma, var, tvs);
    Ok((Subst::new(), ty, expr))
}

fn translate_lam(body: Expr, params: &Vec<TermVar>, params_ty: &Vec<Type>)
                 -> Expr {
    let params  = params
        .iter()
        .zip(params_ty)
        .map( |(v,ty)| v.with_ty(ty.clone()) )
        .collect::<Vec<_>>();
    Expr::Lam(params, Box::new(body))
}

fn infer_lam(mut gamma: Env, params: &Vec<TermVar>, body: &Expr)
             -> Result<(Subst, Type, Expr)>
{
    use self::Type::*;
    let params_ty = params
        .iter()
        .map(| v | {
            let tv = fresh_tyvar();
            gamma.extend(v, ForAll::new(vec![], Var(tv)));
            Var(tv)
        })
        .collect();
    let (s1, t1, body) = infer(&mut gamma, body)?;
    let expr = translate_lam(body, params, &params_ty);
    let fnty = mk_func(&params_ty, t1);
    let fnty = s1.apply(&fnty);
    Ok((s1, fnty, expr))
}

fn infer_app(n: u32, gamma: &mut Env, callee: &Expr, arg: &Expr)
             -> Result<(Subst, Type, Expr)>
{
    let (s1, t1, callee) = infer(gamma, callee)?;
    let mut gamma        = gamma.apply_subst(&s1);
    let (s2, t2, arg)    = infer(&mut gamma, arg)?;
    let retty            = Type::Var(fresh_tyvar());
    let fnty             = mk_fn(t2.clone(), retty.clone(), n+1);
    let s3               = unify(&s2.apply(&t1), &fnty)?;
    let t                = s3.apply(&retty);
    let subst            = s3.compose(&s2)?.
        compose(&s1)?;
    let app              = Expr::App(n, Box::new(callee), Box::new(arg));
    Ok((subst, t, app))
}

fn is_value(expr: &Expr) -> bool {
    use self::Expr::*;
    match *expr {
        UnitLit    |
        BoolLit(_) |
        I32Lit(_)  |
        Lam(_, _)  |
        Var(_)  => true,
        _       => false
    }
}

fn infer_let(gamma: &mut Env, let_exp: &Let) -> Result<(Subst, Type, Expr)>
{
    let (s1, t1, e1) = infer(gamma, let_exp.bind())?;
    let v            = let_exp.id();
    let v            = v.with_ty(t1.clone());
    let mut gamma1   = gamma.apply_subst(&s1);
    // Do value restriction: Don't generalize unless the bind expr is a value
    let t2           = match is_value(let_exp.bind()) {
        true  => generalize(t1, &gamma1),
        false => ForAll::new(vec![], t1)
    };
    gamma1.extend(let_exp.id(), t2.clone());
    let (s2, t, e2)  = infer(&mut gamma1, let_exp.expr())?;
    let s            = s2.compose(&s1)?;
    let tylam        = Expr::TyLam(t2.bound_vars().clone(), Box::new(e1));
    let let_exp      = Let::new(v, tylam, e2);
    let expr         = Expr::Let(Box::new(let_exp));
    Ok((s, t, expr))
}

fn infer_letrec(gamma: &mut Env, v: &TermVar, e: &Expr) 
                -> Result<(Subst, Type, Expr)>
{
    let beta = Type::Var(fresh_tyvar());
    gamma.extend(v, ForAll::new(vec![], beta.clone()));
    let (s1, t1, e) = infer(gamma, e)?;
    let s2 = unify(&beta, &s1.apply(&t1))?;
    let s  = s2.compose(&s1)?;

    //Adds type abstraction to introduce/close over the free type variables
    //   in the body of a lambda. Adds polymorphism to the expression tree
    //e.g. the following gets translated as 
    //   let foo f x = fx;; aka let foo = λf. λy. f x
    //into
    //   let foo = Λ a b. ( λf. λy. f x )
    //
    let t2 = generalize(t1.clone(), &gamma);
    let bv = t2.bound_vars().clone();
    let e  = Expr::TyLam(bv.clone(), Box::new(e));

    gamma.extend(v, ForAll::new(bv, t1.clone()));
    
    Ok((s, t1, e))
}

pub (super) fn infer_fn(gamma: &mut Env, v: &TermVar, e: &Expr) ->
    Result<(Subst, Type, Expr)>
{
    infer_letrec(gamma, v, e)
}

fn infer_if(gamma: &mut Env, if_expr: &If) -> Result<(Subst, Type, Expr)>
{
    let (s1, t1, cond) = infer(gamma, if_expr.cond())?;
    let (s2, t2, texp) = infer(gamma, if_expr.texpr())?;
    let (s3, t3, fexp) = infer(gamma, if_expr.fexpr())?;

    let s4 = unify(&t1, &mk_tycon("bool", 0))?;
    let s5 = unify(&t2, &t3)?;

    let ty = s5.apply(&t2);
    let subst = s5.compose(&s4)?.
        compose(&s3)?.
        compose(&s2)?.
        compose(&s1)?;
    let if_expr = Expr::If(Box::new(If::new(cond, texp, fexp, ty.clone())));
    Ok((subst, ty, if_expr))
}
