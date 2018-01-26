use std::collections::HashMap;
use super::types::{Type,TyVar};
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

    pub fn find(&self, tyvar: TyVar) -> Option<&Type> {
        self.map.get(&tyvar)
    }

    pub fn compose(self, rhs: &Subst) -> Result<Subst> {
        let mut subst = self;
        for (tyvar, ty) in &rhs.map {
            let ty = subst.apply(ty);
            subst.map.insert(*tyvar, ty);
        }
        Ok(subst)
    }
    
    pub fn apply(&self, ty: &Type) -> Type {
        use types::Type::*;
        match *ty {
            Con(ref con, arity)   => Con(con.clone(), arity),
            App(ref con, ref arg) => {
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
