use super::types::{Type, UnboundTyVar, TyVarSubst};
use ::{Error,Result};
use std::rc::Rc;
use std::cell::RefCell;

fn occurs(tv1: UnboundTyVar, ty: &Type) -> bool {
    use types::Type::*;
    match ty {
        Con(_, _)   => false,
        Var(ref tv2) => {
            let unbound = {
                match *tv2.1.borrow() {
                    TyVarSubst::Unbound(tv2) => {
                        if tv1.id == tv2.id { return true };
                        //Move tv2 to the min of its level and tv1's level
                        let level = match tv1.level < tv2.level {
                            true  => tv1.level,
                            false => tv2.level
                        };
                        UnboundTyVar{ id: tv2.id, level }
                    }
                    TyVarSubst::Bound{ ref repr, .. } => {
                        return occurs(tv1, &repr.borrow())
                    }
                }
            };
            *tv2.1.borrow_mut() = TyVarSubst::Unbound(unbound);
            false
        }
        App(ref con, ref args) => {
            args.iter()
                .fold(
                    occurs(tv1, con), |acc, arg| { acc || occurs(tv1, arg) }
                )
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
            let unbound = {
                match *tyvar.1.borrow() {
                    Bound{ ref repr, .. } => return unify(&repr.borrow(), ty),
                    Unbound(unbound) => unbound
                }
            };
            
            if let Var(ref tyvar_rhs) = ty {
                match *tyvar_rhs.1.borrow() {
                    TyVarSubst::Unbound(tyvar) =>
                        if tyvar.id == unbound.id {
                            return Ok(())
                        },
                    _ => ()
                }
            }
            
            if occurs(unbound, ty) {
                let msg = format!("Can not unify {:?} with {:?}", tyvar, ty);
                return Err(Error::new(msg));
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

