use std::fmt;
use std::rc::Rc;

pub mod types;
pub mod subst;
mod unify;
mod hm;
mod env;
mod typecheck;
mod type_env;

pub use self::typecheck::TypeChecker;

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


