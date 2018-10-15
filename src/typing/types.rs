use std::rc::Rc;
use super::subst::Subst;
use std::collections::HashSet;
use super::Kind;
use std::fmt;

#[derive(Copy,Clone,Hash,PartialEq,Eq)]
pub struct TyVar(pub (super) u32);

pub fn fresh_tyvar() -> TyVar {
    TyVar(::fresh_id())
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
    pub fn free_tyvars(&self) -> HashSet<TyVar>
    {
        use self::Type::*;
        let mut res = HashSet::new();
        match *self {
            Con(_, _) => (),
            Var(v)    => {res.insert(v);}
            App(ref con, ref args) => {
                res = con.free_tyvars();
                for arg in args {
                    let arg_ftv = arg.free_tyvars();
                    res = &res | &arg_ftv;
                }
            }
        }
        res
    }
}

impl fmt::Debug for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Type::*;
        match *self {
            Con(ref str, ref k) => write!(f, "{:?}:{:?}", str, k),
            App(ref a, ref b)   => write!(f, "App({:?}, {:?})", a, b),
            Var(v)              => write!(f, "{:?}", v),
        }
    }
}

impl fmt::Debug for TyVar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "'a{}", self.0)
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
    pub fn free_tyvars(&self) -> HashSet<TyVar>
    {
        let mut bound_tv = HashSet::new();
        for v in self.bound_vars() {
            bound_tv.insert(*v);
        }
        let ftv = self.ty.free_tyvars();
        ftv.difference(&bound_tv);
        ftv
    }

    pub (super) fn instantiate(&self, env: &mut super::env::Env) -> (Vec<TyVar>, Type) {
        let mut subst = Subst::new();
        let mut tvs   = Vec::new();
        for bv in &self.bound_vars {
            let tv = env.fresh_tyvar();
            tvs.push(tv);
            subst.bind(*bv, Type::Var(tv));
        }
        (tvs, subst.apply(self.ty()))
    }
    pub fn apply_subst(&mut self, subst: &Subst ) {
        self.ty = subst.apply(self.ty());
    }
}

pub (super) fn generalize(ty: Type, env: &super::env::Env) -> ForAll {
    let ftv1 = ty.free_tyvars();
    let ftv2 = env.free_tyvars();
    let ftv  = ftv1.difference(&ftv2)
        .cloned()
        .collect();
    ForAll::new(ftv, ty)
}
    
