use std::collections::HashMap;
use ::{Result,Error};
use hir::Ident;
use super::types::ForAll;

#[derive(Clone)]
pub (super) struct Env {
    map: HashMap<u32, ForAll<u32>> 
}

impl Env {
    pub fn new() -> Self {
        Self{ map: HashMap::new() }
    }
    
    pub fn lookup(&self, id: &Ident) -> Result<ForAll<u32>> {
        match self.map.get(&id.id()) {
            Some(ref ty) => Ok((*ty).clone()),
            None         => Err(Error::new(format!("Could not find {:?}", id)))
        }
    }

    pub fn extend(&mut self, id: &Ident, ty: ForAll<u32>) {
        self.map.insert(id.id(), ty);
    }

}
