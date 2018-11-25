use std::fmt;
use std::rc::Rc;

use super::{
    Type,
    TVar
};

#[derive(Clone,Hash,PartialEq,Eq)]
pub enum TyCon<T: TVar> {
    NewType(Rc<String>),
    I32,
    Bool,
    Unit,
    Func,
    Record(Rc<Record<T>>),
}

#[derive(Clone,Hash,PartialEq,Eq)]
pub struct NewType<T: TVar> {
    name: Rc<String>,
    args: Vec<T>,
    body: Type<T>,
}

#[derive(Clone,Hash,PartialEq,Eq)]
pub struct Field<T: TVar> {
    name: Rc<String>,
    ty: Type<T>
}

#[derive(Clone,Hash,PartialEq,Eq)]
pub struct Record<T: TVar> {
    name:   Rc<String>,
    fields: Vec<Field<T>>,
}


impl <T: TVar> fmt::Debug for TyCon<T> {
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

impl <T: TVar> fmt::Debug for Field<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:}: {:?}", self.name, self.ty)
    }
}

impl <T: TVar> fmt::Debug for Record<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:}", self.name)?;
        for field in &self.fields {
            write!(f, "{:?}", field)?;
        }
        Ok(())
    }
}
