use super::types::{Type,TyVar};
use super::subst::Subst;
use ::{Error,Result};

fn occurs(tyvar: TyVar, ty: &Type) -> bool
{
    use types::Type::*;
    match *ty {
        Con(_, _) |
        Var(_)    => false,
        ref app @ App(_, _) => {
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
        (&Con(ref l, ref lk), &Con(ref r, ref rk)) => {
            if *l == *r && lk == rk {
                Subst::new()
            } else {
                let msg = format!("Can not unify {:?} with {:?}", lhs, rhs);
                return Err(Error::new(msg));
            }
        }
        (&App(ref lty, ref largs), &App(ref rty, ref rargs)) => {
            if largs.len() != rargs.len() {
                let msg = format!("Can not unify {:?} with {:?}", lhs, rhs);
                return Err(Error::new(msg));
            }
            else {
                let mut sub = unify(lty, rty)?;
                for (larg, rarg) in largs.iter().zip(rargs) {
                    let sub1 = unify(&sub.apply(larg), &sub.apply(rarg))?;
                    sub = sub1.compose(&sub)?;
                }
                sub
            }
        }
        (&Var(tyvar), ty) |
        (ty, &Var(tyvar)) => {
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

