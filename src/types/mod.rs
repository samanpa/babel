mod tvar;
mod tycon;
mod subst;

pub use self::tvar::TyVar;
pub use self::tycon::*;
pub use self::subst::Subst;
use std::collections::HashSet;
use std::fmt;
use std::rc::Rc;


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


#[derive(Clone,Hash,PartialEq,Eq)]
pub enum Type {
    Con(TyCon, Kind),
    App(Box<Type>, Vec<Type>),
    Var(TyVar)
}

impl Type {
    fn free_tyvars(&self, curr_level: u32, res: &mut HashSet<TyVar>) {
        use self::Type::*;
        match *self {
            Con(_, _)  => {}
            Var(ref v) => {
                if v.inner.borrow().level <= curr_level {
                    res.insert(v.clone());
                }
            }
            App(ref con, ref args) => {
                con.free_tyvars(curr_level, res);
                for arg in args {
                    arg.free_tyvars(curr_level, res);
                }
            }
        }
    }

    pub (super) fn generalize(&self, level: u32) -> ForAll {
        let mut tyvars = HashSet::new();
        self.free_tyvars(level, &mut tyvars);
        let ftv = tyvars.into_iter().collect();
        ForAll::new(ftv, self.clone())
    }
}

impl fmt::Debug for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Type::*;
        match *self {
            Con(ref str, ref k) => write!(f, "{:?}:{:?}", str, k),
            App(ref a, ref b)   => write!(f, "App({:?}, {:?})", a, b),
            Var(ref v)          => write!(f, "{:?}", v),
        }
    }
}

#[derive(Debug,Clone,Hash,Eq,PartialEq)]
pub struct ForAll {
    bound_vars: Vec<TyVar>,
    ty: Type
}

impl ForAll {
    pub fn new(bound_vars: Vec<TyVar>, ty: Type) -> Self {
        ForAll{ bound_vars, ty }
    }

    pub fn bound_vars(&self) -> &Vec<TyVar> {
        &self.bound_vars
    }

    pub fn ty(&self) -> &Type {
        &self.ty
    }
}
