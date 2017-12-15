use super::types::Type;
use super::subst::Subst;
use ::{Error,Result};

fn occurs(tyvar: u32, ty: &Type<u32>) -> bool
{
    use types::Type::*;
    match ty {
        &Bool     |
        &I32      |
        &Unit     |
        &TyCon(_) => false,
        &TyVar(_) => false,
        f @ &Func(_) => f.free_tyvars()
            .iter()
            .fold( false, | acc, tv | acc || tyvar == *tv )
    }
}
        
pub fn unify<'a>(lhs: &'a Type<u32>, rhs: &'a Type<u32>) -> Result<Subst>
{
    use types::Type::*;
    let subst = match (lhs, rhs) {
        (&Bool, &Bool)   |
        (&I32,  &I32)    |
        (&Unit, &Unit)   => Subst::new(),
        (&TyVar(tyvar), ref ty) |
        (ref ty, &TyVar(tyvar)) => {
            if occurs(tyvar, &ty) {
                let msg = format!("Can not unify {:?} with {:?}", tyvar, ty);
                return Err(Error::new(msg));
            }
            let mut subst = Subst::new();
            subst.bind(tyvar, (*ty).clone());
            subst
        }
        (&Func(ref lhs), &Func(ref rhs)) => {
            let mut subst = unify(lhs.return_ty(), rhs.return_ty())?;
            if lhs.params_ty().len() != rhs.params_ty().len() {
                let msg = format!("Arity diff \n{:?} with \n{:?}", lhs, rhs);
                return Err(Error::new(msg));
            }
            let mut rhs = rhs.params_ty().iter();
            let mut lhs = lhs.params_ty().iter();
            while let (Some(lhs), Some(rhs)) = (lhs.next(), rhs.next()) {
                let subst1 = unify(lhs, rhs)?;
                subst = subst.compose(&subst1);
            }
            subst
        },
        _ => {
            let msg = format!("Can not unify \n{:?} with \n{:?}", lhs, rhs);
            return Err(Error::new(msg))
        }
    };
    Ok(subst)
}

