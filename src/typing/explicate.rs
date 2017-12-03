use ::types::Type;
use ::hir::Expr;
use ::Result;
use super::subst::Subst;

fn unify(subst: &mut Subst, ty1: &Type<u32>, ty2: &Type<u32>) -> Result<()> {
    Ok(())
}

pub fn infer_ty_args(callee: &Expr
                     , callee_ty: &::types::Function<u32>
                     , args: &Vec<Expr>
                     , subst: &Subst)-> Result<Subst> {
    if callee_ty.is_monotype() {
        return Ok(Subst::new())
    }

    let subst = Subst::new();
    Ok(subst)
}

