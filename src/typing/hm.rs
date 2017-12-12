use super::types::{Type,Function,TyVarFlavour};
use super::subst::Subst;
use super::env::Env;
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
            let ty = gamma.lookup(id)?;
            (subst, ty)
        }
        Lam(ref lam)   => infer_lam(gamma.clone(), lam)?,
        App(ref callee, ref args) => infer_app(gamma, callee, args)?,
        If(ref if_exp) => infer_if(gamma, if_exp)?,
    };
    Ok((subst, ty))
}

fn infer_lam(mut gamma: Env, lam: &Lam) -> Result<(Subst, Type<u32>)> {
    use self::Type::*;
    use self::Expr::*;
    //HACK to handle recursion we should use Y combinator instead
    let fn_ty = TyVar(fresh_id(), TyVarFlavour::Bound);
    gamma.extend(lam.proto().ident(), fn_ty);
    let params_ty = lam.proto().params()
        .iter()
        .map(| id | {
            let tv = fresh_id();
            gamma.extend(id, TyVar(tv, TyVarFlavour::Bound));
            TyVar(tv, TyVarFlavour::Bound)
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
    unimplemented!()
}
