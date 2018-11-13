use std::fmt;
use std::rc::Rc;

use super::{
    TyVar,
    Type
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
