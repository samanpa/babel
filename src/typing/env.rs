use std::collections::HashMap;
use ::{Result,Error};
use hir::Ident;
use super::types::Type;

#[derive(Clone)]
pub (super) struct Env {
    map: HashMap<u32, Type<u32>> 
}

impl Env {
    pub fn new() -> Self {
        Self{ map: HashMap::new() }
    }
    
    pub fn lookup(&self, id: &Ident) -> Result<Type<u32>> {
        match self.map.get(&id.id()) {
            Some(ref ty) => Ok((*ty).clone()),
            None         => Err(Error::new(format!("Could not find {:?}", id)))
        }
    }

    pub fn extend(&mut self, id: &Ident, ty: Type<u32>) {
        self.map.insert(id.id(), ty);
    }

}
