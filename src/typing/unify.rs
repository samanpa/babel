use super::types::{Type,TyVar, TyVarSubst};
use ::{Error,Result};
use std::rc::Rc;
use std::cell::RefCell;

fn occurs(tyvar: &TyVar, ty: &Type) -> bool {
    use types::Type::*;
    match *ty {
        Con(_, _) |
        Var(_)    => false,
        ref app @ App(_, _) => {
            app.free_tyvars()
                .iter()
                .fold( false, | acc, tv | acc || *tyvar == *tv )
        }
    }
}

pub fn unify<'a>(lhs: &'a Type, rhs: &'a Type) -> Result<()> {
    use types::Type::*;
    match (lhs, rhs) {
        (&Con(ref l, ref lk), &Con(ref r, ref rk)) => {
            if !(*l == *r && lk == rk) {
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
                unify(lty, rty)?;
                for (larg, rarg) in largs.iter().zip(rargs) {
                    unify(&larg.apply_subst(), &rarg.apply_subst())?;
                }
            }
        }
        (&Var(ref tyvar), ty) |
        (ty, &Var(ref tyvar)) => {
            //println!("{:?} -> {:?}", tyvar, ty);

            use self::TyVarSubst::*;
            if let Bound{ ref repr, .. } =  *tyvar.1.borrow() {
                return unify(&repr.borrow(), ty)
            }


            if occurs(tyvar, ty) {
                let msg = format!("Can not unify {:?} with {:?}", tyvar, ty);
                return Err(Error::new(msg));
            }
            if let Var(ref tyvar_rhs) = ty {
                if tyvar_rhs.0 == tyvar.0 {
                    return Ok(())
                }
            }
            let bound = Bound {
                rank: 0,
                repr: Rc::new(RefCell::new(ty.clone()))
            };
            *tyvar.1.borrow_mut() = bound;
        }
        _ => {
            let msg = format!("Can not unify\n\t- {:?} \n\t- {:?}", lhs, rhs);
            return Err(Error::new(msg))
        }
    };
    Ok(())
}

