use std::collections::{HashMap,HashSet};
use ::{Result,Error};
use hir::Ident;
use super::types::ForAll;
use super::subst::Subst;

#[derive(Clone,Debug)]
pub (super) struct Env {
    map: HashMap<u32, ForAll> 
}

impl Env {
    pub fn new() -> Self {
        Self{ map: HashMap::new() }
    }
    
    pub fn lookup(&self, id: &Ident) -> Result<ForAll> {
        match self.map.get(&id.id()) {
            Some(ref ty) => Ok((*ty).clone()),
            None         => Err(Error::new(format!("Could not find {:?}", id)))
        }
    }

    pub fn extend(&mut self, id: &Ident, ty: ForAll) {
        self.map.insert(id.id(), ty);
    }

    pub fn apply_subst(&self, sub: &Subst) -> Env {
        let mut env = self.clone();
        for val in env.map.values_mut() {
            val.apply_subst(sub)
        }
        env
    }

    pub fn free_tyvars(&self) -> HashSet<u32>{
        let mut ftv = HashSet::new();
        for (_,sigma) in &self.map {
            ftv = &ftv | &sigma.free_tyvars();
        }
        ftv
    }

}
