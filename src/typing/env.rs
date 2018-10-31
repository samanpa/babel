use std::collections::{HashMap};
use ::{Result,Error};
use idtree::Symbol;
use super::types::{ForAll};

#[derive(Clone,Debug)]
pub (super) struct Env {
    map: HashMap<u32, ForAll>
}

impl Env {
    pub fn new() -> Self {
        Self{ map: HashMap::new() }
    }
    
    pub fn lookup(&self, id: &Symbol) -> Result<ForAll> {
        match self.map.get(&id.id()) {
            Some(ty) => Ok(ty.clone()),
            None     => Err(Error::new(format!("Could not find {:?}", id)))
        }
    }

    pub fn extend(&mut self, id: &Symbol, ty: ForAll) {
        self.map.insert(id.id(), ty);
    }

    pub fn apply_subst(&self) -> Env {
        let mut env = self.clone();
        for val in env.map.values_mut() {
            val.apply_subst()
        }
        env
    }
}
