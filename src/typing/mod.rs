use std::fmt;
use std::rc::Rc;

pub mod types;
pub mod subst;
mod unify;
mod hm;
mod env;
mod typecheck;

pub use self::typecheck::TypeChecker;

#[derive(Clone,Hash,PartialEq,Eq)]
pub enum Kind {
    Star,
    Fun(Rc<Kind>, Rc<Kind>)
}

impl fmt::Debug for Kind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Kind::Star              => write!(f, "*"),
            Kind::Fun(ref l, ref r) => write!(f, "{:?} => {:?}", *l, *r),
        }
    }
}


