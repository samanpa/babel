use std::hash::{Hash, Hasher};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone,PartialEq,Eq)]
pub struct InnerTyVar {
    pub level: u32
}
    
#[derive(Clone,PartialEq,Eq)]
pub struct TyVar {
    pub id: u32,
    pub inner: Rc<RefCell<InnerTyVar>>
}

impl Hash for TyVar {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl TyVar {
    pub fn fresh(level: u32) -> TyVar {
        let inner = InnerTyVar{ level };
        TyVar{
            id: ::fresh_id(),
            inner: Rc::new(RefCell::new(inner))
        }
    }
}

