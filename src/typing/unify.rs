use super::types::{Type,TyVar, TyVarSubst};
use ::{Error,Result};
use std::rc::Rc;
use std::cell::RefCell;

fn occurs(tyvar: &TyVar, lvl: u32, ty: &Type) -> bool {
    use types::Type::*;
    match *ty {
        Con(_, _)   => false,
        Var(ref tv) => {
            use self::TyVarSubst::*;
            let min_level = {
                match *tv.1.borrow() {
                    Unbound{ level } => {
                        if tyvar.0 == tv.0 { return true };
                        if level < lvl { level } else { lvl }
                    }
                    Bound{ ref repr, .. } => {
                        return occurs(tyvar, lvl, &repr.borrow())
                    }
                }
            };
            *tv.1.borrow_mut() = Unbound{ level: min_level };
            false
        }
        App(ref con, ref args) => {
            args.iter()
                .fold( occurs(tyvar, lvl, con), |acc, arg| {
                    acc || occurs(tyvar, lvl, arg)
                })
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
            let level = {
                match *tyvar.1.borrow() {
                    Bound{ ref repr, .. } => return unify(&repr.borrow(), ty),
                    Unbound{ level } => level
                }
            };
            
            if let Var(ref tyvar_rhs) = ty {
                if tyvar_rhs.0 == tyvar.0 {
                    return Ok(())
                }
            }
            
            if occurs(tyvar, level, ty) {
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

