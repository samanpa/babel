use std::collections::HashMap;
use super::types::{Type,TyVar};
use super::unify::unify;
use std::collections::hash_map::Entry;
use ::Result;

#[derive(Debug)]
pub struct Subst {
    map: HashMap<TyVar,Type>,
}

impl Default for Subst {
    fn default() -> Self {
        Self::new()
    }
}

impl Subst {
    pub fn new() -> Self {
        Subst{ map: HashMap::new() }
    }

    pub fn bind(&mut self, tyvar: TyVar, ty: Type) {
        self.map.insert(tyvar, ty);
    }

    pub fn compose(self, rhs: &Subst) -> Result<Subst> {
        let mut subst = self;
        for (tyvar, ty) in &rhs.map {
            let ty = subst.apply(ty);
            let new_subst = match subst.map.entry(*tyvar) {
                Entry::Vacant(elem)  => {
                    elem.insert(ty);
                    None
                }
                Entry::Occupied(entry) => {
                    let res = unify(entry.get(), &ty)?;
                    Some(res)
                }
            };
            if let Some(new_subst) = new_subst {
                //Handle the case where both lhs and rhs have the same
                // binding e.g
                //   lhs a1: i32 -> a3
                //   rhs a1: i32 -> i32
                // This should introduce a new binding
                //   a3: i32
                for (tyvar, ty) in new_subst.map {
                    subst.map.insert(tyvar, ty);
                }
            }
        }
        Ok(subst)
    }
    
    pub fn apply(&self, ty: &Type) -> Type {
        use types::Type::*;
        match *ty {
            Con(ref con, ref kind) => Con(con.clone(), kind.clone()),
            App(ref con, ref arg)  => {
                let con = self.apply(con);
                let arg = self.apply(arg);
                App(Box::new(con), Box::new(arg))
            }
            Var(id) => {
                match self.map.get(&id) {
                    Some(ty) => ty.clone(),
                    None     => Var(id),
                }
            }
        }
    }
}
