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

pub trait TVar : fmt::Debug {}

impl TVar for String {}

#[derive(Clone,Hash,PartialEq,Eq)]
pub enum Type<T: TVar> {
    App(Box<Type<T>>, Vec<Type<T>>),
    Con(TyCon<T>, Kind),
    Var(T)
}

#[derive(Debug,Clone,Hash,Eq,PartialEq)]
pub struct ForAll<T: TVar> {
    bound_vars: Vec<T>,
    ty: Type<T>
}


impl fmt::Debug for Kind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Kind::Star       => write!(f, "*"),
            Kind::Fun(ref k) => write!(f, "{:?} => {:?}", k.0, k.1),
        }
    }
}

impl <T: TVar> Type<T> {
    pub fn func(mut params: Vec<Type<T>>, ret: Type<T>) -> Type<T> {
        use self::Type::*;
        use self::Kind::*;
        let mk_kind = |n| {
            (0..(n+1))
                .fold( Star, |kind, _| Fun(Rc::new((Star, kind))))
        };
        let con = Con(TyCon::Func, mk_kind(params.len()));
        params.push(ret);
        App(Box::new(con), params)
    }

    pub fn unit() -> Type<T> {
        Type::Con(TyCon::Unit, Kind::Star)
    }
}


impl Type<TyVar> {
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

    pub (super) fn generalize(&self, level: u32) -> ForAll<TyVar> {
        let mut tyvars = HashSet::new();
        self.free_tyvars(level, &mut tyvars);
        let ftv = tyvars.into_iter().collect();
        ForAll::new(ftv, self.clone())
    }
}

impl <T: TVar> fmt::Debug for Type<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Type::*;
        match *self {
            Con(ref str, ref k) => write!(f, "{:?}:{:?}", str, k),
            App(ref a, ref b)   => write!(f, "App({:?}, {:?})", a, b),
            Var(ref v)          => write!(f, "{:?}", v),
        }
    }
}

impl <T: TVar> ForAll<T> {
    pub fn new(bound_vars: Vec<T>, ty: Type<T>) -> Self {
        ForAll{ bound_vars, ty }
    }

    pub fn bound_vars(&self) -> &Vec<T> {
        &self.bound_vars
    }

    pub fn ty(&self) -> &Type<T> {
        &self.ty
    }
}
