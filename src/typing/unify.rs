use super::types::Type;
use super::subst::Subst;
use ::{Error,Result};

fn occurs(tyvar: u32, ty: &Type) -> bool
{
    use types::Type::*;
    match *ty {
        TyCon(_) |
        TyVar(_) => false,
        ref app @ TyApp(_, _) => {
            app.free_tyvars()
                .iter()
                .fold( false, | acc, tv | acc || tyvar == *tv )
        }
    }
}

pub fn unify<'a>(lhs: &'a Type, rhs: &'a Type) -> Result<Subst>
{
    use types::Type::*;
    let subst = match (lhs, rhs) {
        (&TyCon(ref l), &TyCon(ref r)) => {
            if *l == *r {
                Subst::new()
            } else {
                let msg = format!("Can not unify {:?} with {:?}", l, r);
                return Err(Error::new(msg));
            }
        }
        (&TyApp(ref lty, ref larg), &TyApp(ref rty, ref rarg)) => {
            if larg.len() != rarg.len() {
                let msg = format!("Arity diff \n{:?} with \n{:?}", lty, rty);
                return Err(Error::new(msg));
            }
            
            let mut sub = unify(lty, rty)?;
            for (l, r) in larg.iter().zip(rarg.iter()) {
                let sub1 = unify(& sub.apply(l), & sub.apply(r))?;
                sub = sub1.compose(&sub)?;
            }
            sub
        }
        (&TyVar(tyvar), ty) |
        (ty, &TyVar(tyvar)) => {
            if occurs(tyvar, ty) {
                let msg = format!("Can not unify {:?} with {:?}", tyvar, ty);
                return Err(Error::new(msg));
            }
            let mut subst = Subst::new();
            subst.bind(tyvar, (*ty).clone());
            subst
        }
        _ => {
            let msg = format!("Can not unify\n\t- {:?} \n\t- {:?}", lhs, rhs);
            return Err(Error::new(msg))
        }
    };
    Ok(subst)
}

