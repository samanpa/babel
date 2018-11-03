use std::cell::RefCell;
use std::collections::HashSet;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use super::subst::Subst;
use super::Kind;

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

pub fn fresh_tyvar(level: u32) -> TyVar {
    let inner = InnerTyVar{ level };
    TyVar{
        id: ::fresh_id(),
        inner: Rc::new(RefCell::new(inner))
    }
}


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

#[derive(Debug,Clone,Hash,Eq,PartialEq)]
pub struct ForAll {
    bound_vars: Vec<TyVar>,
    ty: Type
}

impl Type {
    pub fn free_tyvars(&self, curr_level: u32, res: &mut HashSet<TyVar>) {
        use self::Type::*;
        match *self {
            Con(_, _)  => (),
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
    pub fn is_monotype(&self) -> bool {
        self.bound_vars.is_empty()
    }
    
    pub (super) fn instantiate(
        &self,
        env: &mut super::env::Env,
        level: u32
    ) -> (Vec<TyVar>, Type) {
        //FIXME: does this even make sense
        let mut subst = Subst::new();
        let mut tvs   = Vec::new();
        for bv in &self.bound_vars {
            let tv = env.fresh_tyvar(level);
            tvs.push(tv.clone());
            subst.bind(bv, Type::Var(tv));
        }
        (tvs, subst.apply(self.ty()))
    }
    pub fn apply_subst(&mut self, subst: &Subst ) {
        self.ty = subst.apply(self.ty());
    }
}

pub (super) fn generalize(ty: Type, level: u32) -> ForAll {
    let mut tyvars = HashSet::new();
    ty.free_tyvars(level, &mut tyvars);
    let ftv  = tyvars.into_iter().collect();
    ForAll::new(ftv, ty)
}

