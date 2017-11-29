use ::types::Type;
use ::hir::Expr;
use ::Result;

pub fn infer_app(callee: &Expr, args: &Vec<Expr>
                 , ty_vars: &Vec<Type<u32>>)-> Result<Expr> {
    Ok(Expr::UnitLit)
}

