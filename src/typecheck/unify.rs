use super::Type;
use crate::types::TyVar;
use crate::utils::{DisjointSet, DisjointSetValue};
use crate::Error;
use std;
use std::collections::HashMap;

#[derive(Debug)]
pub(super) struct UnificationTable {
    subst: DisjointSet<u32, Type>,
    indices: HashMap<u32, u32>,
}

impl DisjointSetValue for Type {}

fn occurs(tv1: &TyVar, ty: &Type, top_level: bool) -> bool {
    use crate::types::Type::*;
    match *ty {
        Con(_, _) => false,
        Var(ref tv2) => {
            if tv1.id == tv2.id && !top_level {
                true
            } else {
                let tv1 = tv1.inner.borrow();
                let mut tv2 = tv2.inner.borrow_mut();
                //Move tv2 to the min of its level and tv1's level
                let level = std::cmp::min(tv1.level, tv2.level);
                tv2.level = level;
                false
            }
        }
        App(ref con, ref args) => args.iter().fold(occurs(tv1, con, false), |acc, arg| {
            acc || occurs(tv1, arg, false)
        }),
    }
}

impl UnificationTable {
    pub fn new() -> Self {
        let subst = DisjointSet::with_capacity(100);
        let indices = HashMap::new();
        UnificationTable { subst, indices }
    }

    pub fn add(&mut self, tyvar: TyVar) {
        let key = self.subst.add(Type::Var(tyvar.clone()));
        self.indices.insert(tyvar.id, key);
    }

    pub fn apply_subst(&mut self, ty: &Type) -> Type {
        use crate::types::Type::*;
        let res = match ty {
            Con(ref con, ref kind) => Con(con.clone(), kind.clone()),
            App(ref con, ref args) => {
                let con = self.apply_subst(con);
                let args = args.iter().map(|arg| self.apply_subst(arg)).collect();
                App(Box::new(con), args)
            }
            Var(TyVar { id, .. }) => {
                let key = *self.indices.get(id).unwrap();
                let ty = self.subst.find(key).clone();
                match ty {
                    Var(ty) => Var(ty),
                    ty => self.apply_subst(&ty),
                }
            }
        };

        //println!("APPLY SUBST \n{:?} = {:?} \n{:#?}", ty, res, self);
        res
    }

    pub fn unify<'a>(&mut self, lhs: &'a Type, rhs: &'a Type) -> crate::Result<()> {
        use crate::types::Type::*;
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
            (&Var(ref tyvar), ty) | (ty, &Var(ref tyvar)) => {
                if occurs(tyvar, ty, true) {
                    return cannot_unify(&Var(tyvar.clone()), ty);
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

fn cannot_unify(lhs: &Type, rhs: &Type) -> crate::Result<()> {
    let msg = format!("Can not unify {:?} with {:?}", lhs, rhs);
    Err(Error::new(msg))
}
