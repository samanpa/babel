use ::types::Type;
use ::hir::Expr;
use ::Result;

struct Subst {}

fn unify(subst: &mut Subst, ty1: &Type<u32>, ty2: &Type<u32>) -> Result<()> {
    Ok(())
}

pub fn infer_ty_args(callee: &Expr
                     , callee_ty: &::types::Function<u32>
                     , args: &Vec<Expr>
                     , ty_vars: &Vec<Type<u32>>)-> Result<Vec<Type<u32>>> {
    if callee_ty.ty_vars().len() == 0 {
        return Ok(ty_vars.clone())
    }
    
    let ty_vars = ty_vars.clone();
    Ok(ty_vars)
}

