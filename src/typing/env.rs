use std::collections::{HashMap,HashSet};
use ::{Result,Error};
use xir::Ident;
use super::types::{mk_tyvar,TyVar,ForAll};
use super::subst::Subst;

#[derive(Clone,Debug)]
pub (super) struct Env {
    map: HashMap<TyVar, ForAll> 
}

impl Env {
    pub fn new() -> Self {
        Self{ map: HashMap::new() }
    }
    
    pub fn lookup(&self, id: &Ident) -> Result<ForAll> {
        match self.map.get(&mk_tyvar(id.id())) {
            Some(ty) => Ok(ty.clone()),
            None     => Err(Error::new(format!("Could not find {:?}", id)))
        }
    }

    pub fn extend(&mut self, id: &Ident, ty: ForAll) {
        self.map.insert(mk_tyvar(id.id()), ty);
    }

    pub fn apply_subst(&self, sub: &Subst) -> Env {
        let mut env = self.clone();
        for val in env.map.values_mut() {
            val.apply_subst(sub)
        }
        env
    }

    pub fn free_tyvars(&self) -> HashSet<TyVar>{
        let mut ftv = HashSet::new();
        for sigma in self.map.values() {
            ftv.extend(&sigma.free_tyvars());
        }
        ftv
    }

}
