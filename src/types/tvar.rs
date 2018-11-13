use std::hash::{Hash, Hasher};
use std::cell::RefCell;
use std::rc::Rc;
use std::fmt;

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

impl fmt::Debug for TyVar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "'a{}", self.id)
    }
}
