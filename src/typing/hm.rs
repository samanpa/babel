//Standard Hindley Milner augmented with value restriction
//    "Simple imperative polymorphism" - Wright

use super::Kind;
use super::types::{TyCon,Type,TyVar,ForAll,generalize};
use super::env::Env;
use ::idtree;
use ::xir;
use ::Result;
use std::rc::Rc;

pub fn mk_func(mut params: Vec<Type>, ret: Type) -> Type {
    use self::Type::*;
    use self::Kind::*;
    let mk_kind = |n| {
        (0..(n+1))
            .fold( Star, |kind, _| Fun(Rc::new((Star, kind))))
    };
    let con = Con(TyCon::Func, mk_kind(params.len()));
    params.push(ret);
    App(Box::new(con), params)
}

pub (super) fn infer(gamma: &mut Env, expr: &idtree::Expr) -> Result<(Type, xir::Expr)> {
    use self::Kind::*;
    use self::TyCon::*;
    use idtree::Expr::*;

    let (ty, expr) = match *expr {
        UnitLit       => (Type::Con(Unit, Star), xir::Expr::UnitLit),
        I32Lit(n)     => (Type::Con(I32, Star),  xir::Expr::I32Lit(n)),
        BoolLit(b)    => (Type::Con(Bool, Star), xir::Expr::BoolLit(b)),
        Var(ref v)    => infer_var(gamma, v)?,
        If(ref exp)   => infer_if(gamma, exp)?,
        Let(ref exp)  => infer_let(gamma, exp)?,
        App(ref callee, ref args) => infer_app(gamma, callee, args)?,
        Lam(ref proto, ref body)  => infer_lam(gamma.clone(), proto, body)?,
    };
    Ok((ty, expr))
}

pub fn into_xir_symbol(var: &idtree::Symbol, ty: &Type) -> xir::Symbol {
    xir::Symbol::new(var.name().clone(), ty.clone(), var.id())
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
fn translate_var(sigma: &ForAll, var: &idtree::Symbol, tvs: Vec<TyVar>) -> xir::Expr {
    use xir::Expr::*;
    let ty_args = tvs.iter()
        .map( |tv| Type::Var(*tv) )
        .collect::<Vec<_>>();
    let var = into_xir_symbol(var, sigma.ty());
    let var = xir::Expr::Var(var);
    match ty_args.len() {
        0 => var,
        _ => TyApp(Box::new(var), ty_args)
    }
}

fn infer_var(gamma: &mut Env, var: &idtree::Symbol) -> Result<(Type, xir::Expr)> {
    let sigma     = gamma.lookup(var)?;
    let (tvs, ty) = sigma.instantiate(gamma);
    let expr      = translate_var(&sigma, var, tvs);
    Ok((ty, expr))
}

fn translate_lam(body: xir::Expr, params: &Vec<idtree::Symbol>, params_ty: &Vec<Type>, retty: Type)
                 -> xir::Expr {
    let params  = params
        .iter()
        .zip(params_ty)
        .map( |(v,ty)| into_xir_symbol(v, ty) )
        .collect::<Vec<_>>();
    xir::Expr::Lam(params, Box::new(body), retty)
}

fn infer_lam(
    mut gamma: Env,
    params: &Vec<idtree::Symbol>,
    body: &idtree::Expr
) -> Result<(Type, xir::Expr)>
{
    use self::Type::*;
    let params_ty = params
        .iter()
        .map(| v | {
            let tv = gamma.fresh_tyvar();
            gamma.extend(v, ForAll::new(vec![], Var(tv)));
            Var(tv)
        })
        .collect();
    let (t1, body) = infer(&mut gamma, body)?;
    let expr = translate_lam(body, params, &params_ty, t1.clone());
    let fnty = mk_func(params_ty, t1);
    let fnty = gamma.apply(&fnty);
    Ok((fnty, expr))
}

fn infer_args(
    gamma: &mut Env,
    args: &[idtree::Expr]
) -> Result<(Vec<Type>, Vec<xir::Expr>)>
{
    let mut tys   = Vec::with_capacity(args.len());
    let mut exprs = Vec::with_capacity(args.len());

    for arg in args {
        let (t1, arg) = infer(gamma, arg)?;
        tys.push(t1);
        exprs.push(arg);
    }
    Ok((tys, exprs))
}

fn infer_app(
    gamma: &mut Env,
    caller: &idtree::Expr,
    args: &[idtree::Expr]
) -> Result<(Type, xir::Expr)>
{
    let (t1, caller) = infer(gamma, caller)?;
    let retty        = Type::Var(gamma.fresh_tyvar());
    let (t2, args)   = infer_args(gamma, args)?;
    let fnty         = mk_func(t2, retty.clone());
    gamma.unify(&t1, &fnty)?;
    let t            = gamma.apply(&retty);
    let app          = xir::Expr::App(Box::new(caller), args);
    Ok((t, app))
}

fn is_value(expr: &idtree::Expr) -> bool {
    use idtree::Expr::*;
    match *expr {
        UnitLit    |
        BoolLit(_) |
        I32Lit(_)  |
        Lam(_, _)  |
        Var(_)  => true,
        _       => false
    }
}

fn infer_let(
    gamma: &mut Env,
    let_exp: &idtree::Let
) -> Result<(Type, xir::Expr)>
{
    let bind       = let_exp.bind();
    let (t1, e1)   = infer(gamma, bind.expr())?;
    
    let name       = into_xir_symbol(bind.symbol(), &t1);
    let mut gamma1 = gamma.apply_subst();
    // Do value restriction: Don't generalize unless the bind expr is a value
    let t2         = match is_value(bind.expr()) {
        true  => generalize(t1, &gamma1),
        false => ForAll::new(vec![], t1)
    };
    gamma1.extend(bind.symbol(), t2.clone());
    let (t, e2)    = infer(&mut gamma1, let_exp.expr())?;
    let tylam      = xir::Expr::TyLam(t2.bound_vars().clone(), Box::new(e1));
    let let_exp    = xir::Let::new(xir::Bind::non_rec(name, tylam), e2);
    let expr       = xir::Expr::Let(Box::new(let_exp));

    Ok((t, expr))
}

fn infer_recbind(
    gamma: &mut Env,
    v: &idtree::Symbol,
    e: &idtree::Expr
) -> Result<xir::Bind>
{
    //Typing let rec x = e is done by translating it to
    //    let x  = Y (λx.e) where Y is the fixed point combinator.
    // 
    //    W(Γ,x) = let (S1,τ1) = W(Γ, λx.e)
    //                 τ2      = fresh β
    //                 S2      = unify(τ2 → τ2, τ1)
    //             in  (S2, S1 τ2)
    //
    //  We simplify the above and do this instead
    //    W(Γ,x) = let (S1,τ1) = W(Γ ∪ {x: β}, e) where β is fresh
    //                 S2      = unify(β, τ1)
    //             in  (S2 ◦ S1, τ1)

    let beta = Type::Var(gamma.fresh_tyvar());
    gamma.extend(v, ForAll::new(vec![], beta.clone()));
    let (t1, e) = infer(gamma, e)?;
    gamma.unify(&beta, &t1)?;

    //Adds type abstraction to introduce/close over the free type variables
    //   in the body of a lambda. This adds polymorphism to the expression tree
    //e.g. the following gets translated as 
    //   let foo f x = fx;; aka let foo = λf. λy. f x
    //into
    //   let foo = Λ a b. ( λf. λy. f x )
    //
    let t2 = generalize(t1.clone(), &gamma);
    let bv = t2.bound_vars().clone();
    let e  = xir::Expr::TyLam(bv.clone(), Box::new(e));

    gamma.extend(v, ForAll::new(bv, t1.clone()));

    let name  = into_xir_symbol(v, &t1);
    Ok(xir::Bind::NonRec{symbol: name, expr: e})
}

pub (super) fn infer_fn(
    gamma: &mut Env,
    bind: &idtree::Bind
) -> Result<xir::Bind>
{
    infer_recbind(gamma, bind.symbol(), bind.expr())
}

fn infer_if(
    gamma: &mut Env,
    if_expr: &idtree::If
) -> Result<(Type, xir::Expr)>
{
    let (t1, cond) = infer(gamma, if_expr.cond())?;
    let mut gamma  = gamma.apply_subst();
    let (t2, texp) = infer(&mut gamma, if_expr.texpr())?;
    let (t3, fexp) = infer(&mut gamma, if_expr.fexpr())?;

    gamma.unify(&t1, &Type::Con(TyCon::Bool, Kind::Star))?;
    gamma.unify(&t2, &t3)?;

    let ty = gamma.apply(&t2);
    let if_expr = xir::Expr::If(Box::new(xir::If::new(cond, texp, fexp, ty.clone())));
    Ok((ty, if_expr))
}
