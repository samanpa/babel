use std::collections::HashSet;
use std::fmt;
use std::rc::Rc;

use super::{
    ForAll,
    Kind,
    TyVar
};

#[derive(Clone,Hash,PartialEq,Eq)]
pub enum TyCon {
    NewType(Rc<String>),
    I32,
    Bool,
    Unit,
    Func,
    Record(Rc<Record>),
}

#[derive(Clone,Hash,PartialEq,Eq)]
pub struct NewType {
    name: Rc<String>,
    args: Vec<TyVar>,
    body: Type,
}

#[derive(Clone,Hash,PartialEq,Eq)]
pub struct Field {
    name: Rc<String>,
    ty: Type
}

#[derive(Clone,Hash,PartialEq,Eq)]
pub struct Record {
    name:   Rc<String>,
    fields: Vec<Field>,
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

impl fmt::Debug for TyVar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "'a{}", self.id)
    }
}

impl fmt::Debug for TyCon {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::TyCon::*;
        let res;
        let v = match *self {
            I32   => "i32",
            Bool  => "bool",
            Unit  => "()",
            Func  => "->",
            NewType(ref nm) => nm.as_str(),
            Record(ref rec) => { 
                res = format!("{:?}", rec);
                &res
            }
        };
        write!(f, "{}", v)
    }
}

impl fmt::Debug for Field {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:}: {:?}", self.name, self.ty)
    }
}

impl fmt::Debug for Record {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:}", self.name)?;
        for field in &self.fields {
            write!(f, "{:?}", field)?;
        }
        Ok(())
    }
}
