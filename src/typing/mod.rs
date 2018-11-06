use std::fmt;
use std::rc::Rc;

mod types;
mod subst;
mod hm;
mod env;
mod typecheck;
mod unify;
mod scheme;
mod tvar;

pub use self::typecheck::TypeChecker;
pub use self::types::*;
use self::scheme::ForAll;
pub use self::tvar::TyVar;
pub use self::subst::Subst;

#[derive(Clone,Hash,PartialEq,Eq)]
pub enum Kind {
    Star,
    Fun(Rc<(Kind, Kind)>)
}


impl fmt::Debug for Kind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Kind::Star       => write!(f, "*"),
            Kind::Fun(ref k) => write!(f, "{:?} => {:?}", k.0, k.1),
        }
    }
}


