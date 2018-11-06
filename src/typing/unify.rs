use union_find::{DisjointSet, self};
use super::{Type, TyVar};
use std::collections::HashMap;
use ::Error;
use std;

#[derive(Debug)]
pub (super) struct UnificationTable {
    subst: DisjointSet<u32, Type>,
    indices: HashMap<u32, u32>
}

impl union_find::Value for Type {}

impl UnificationTable {
    pub fn new() -> Self {
        let subst = DisjointSet::with_capacity(100);
        let indices = HashMap::new();
        UnificationTable{ subst, indices }
    }

    pub fn add(&mut self, tyvar: TyVar) {
        let key   = self.subst.add(Type::Var(tyvar.clone()));
        self.indices.insert(tyvar.id, key);
    }

    pub fn reset(&mut self) {
        self.subst.reset();
        self.indices.clear();
    }

    pub fn apply_subst(&mut self, ty: &Type) -> Type {
        use super::Type::*;
        let res = match ty {
            Con(ref con, ref kind) => Con(con.clone(), kind.clone()),
            App(ref con, ref args)  => {
                let con = self.apply_subst(con);
                let args = args.iter()
                    .map( |arg| self.apply_subst(arg))
                    .collect();
                App(Box::new(con), args)
            }
            Var(TyVar{id, ..}) => {
                let key = *self.indices.get(id).unwrap();
                let ty = self.subst.find(key).clone();
                match ty {
                    Var(ty) => Var(ty),
                    ty      => self.apply_subst(&ty)
                }
            }
        };

        //println!("APPLY SUBST \n{:?} = {:?} \n{:#?}", ty, res, self);
        res
    }

    pub fn unify<'a>(&mut self, lhs: &'a Type, rhs: &'a Type) -> ::Result<()> {
        use super::Type::*;
        match (lhs, rhs) {
            (&Con(ref l, ref lk), &Con(ref r, ref rk)) => {
                if *l != *r || lk != rk {
                    return cannot_unify(lhs, rhs);
                }
            }
            (&App(ref lty, ref largs), &App(ref rty, ref rargs)) => {
                if largs.len() != rargs.len() {
                    return cannot_unify(lhs, rhs);
                }
                self.unify(lty, rty)?;
                for (larg, rarg) in largs.iter().zip(rargs) {
                    self.unify(larg, rarg)?;
                }
            }
            (&Var(ref tyvar1), &Var(ref tyvar2)) => {
                let key1 = *self.indices.get(&tyvar1.id).unwrap();
                let key2 = *self.indices.get(&tyvar2.id).unwrap();
                self.subst.merge(key1, key2);
            }
            (&Var(ref tyvar), ty) |
            (ty, &Var(ref tyvar)) => {
                if let Var(tv2) = ty {
                    if tv2.id == tyvar.id {
                        return Ok(())
                    }
                }
                if occurs(tyvar, ty) {
                    return cannot_unify(&Type::Var(tyvar.clone()), ty);
                }
                let old = {
                    let key = *self.indices.get(&tyvar.id).unwrap();
                    let new_ty = self.subst.find(key);
                    std::mem::replace(new_ty, ty.clone())
                };
                self.unify(&old, ty)?;
            }
            _ => {
                return cannot_unify(lhs, rhs);
            }
        };
        Ok(())
    }
}

fn cannot_unify( lhs: &Type, rhs: &Type) -> ::Result<()> {
    let msg = format!("Can not unify {:?} with {:?}", lhs, rhs);
    return Err(Error::new(msg));
}

fn occurs(tv1: &TyVar, ty: &Type) -> bool {
    use super::Type::*;
    match *ty {
        Con(_, _) => false,
        Var(ref tv2) => {
            if tv1.id == tv2.id {
                true
            }
            else {
                let tv1 = tv1.inner.borrow();
                let mut tv2 = tv2.inner.borrow_mut();
                //Move tv2 to the min of its level and tv1's level
                let level = match tv1.level < tv2.level {
                    true  => tv1.level,
                    false => tv2.level
                };
                tv2.level = level;
                false
            }
        },
        App(ref con, ref args) => {
            args.iter()
                .fold(
                    occurs(tv1, con),
                    |acc, arg| { acc || occurs(tv1, arg) }
                )
        }
    }
}
