use std::{
    cell::RefCell,
    collections::HashSet,
    fmt,
    hash::{
        Hash,
        Hasher
    },
    rc::Rc
};
use super::subst::Subst;
use super::Kind;

#[derive(Clone,PartialEq,Eq)]
pub enum TyVarSubst {
    Unbound{ level: u32 },
    Bound{ rank: u32, repr: Rc<RefCell<Type>> }
}
         
#[derive(Clone,PartialEq,Eq)]
pub struct TyVar(pub (super) u32, pub (super) Rc<RefCell<TyVarSubst>>);

impl Hash for TyVar {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

pub fn fresh_tyvar(level: u32) -> TyVar {
    let subst = TyVarSubst::Unbound{ level };
    let rc = Rc::new(RefCell::new(subst));
    TyVar(::fresh_id(), rc)
}

#[derive(Clone,Hash,PartialEq,Eq)]
pub enum TyCon {
    Cus(Rc<String>),
    I32,
    Bool,
    Unit,
    Func,
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
    pub fn free_tyvars(&self) -> HashSet<TyVar> {
        use self::Type::*;
        let mut res = HashSet::new();
        match *self {
            Con(_, _) => (),
            Var(ref v)    => {res.insert(v.clone());}
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

    pub (super) fn apply_subst(&self) -> super::types::Type {
        use super::types::TyVarSubst::*;
        use super::types::Type::*;
        match *self {
            Var(ref tyvar) => {
                match *tyvar.1.borrow() {
                    Unbound{..} => self.clone(),
                    Bound{ ref repr, .. } => repr.borrow().apply_subst()
                }
            }
            App(ref con, ref args)  => {
                let con = con.apply_subst();
                let args = args.iter()
                    .map( |arg| arg.apply_subst())
                    .collect();
                App(Box::new(con), args)
            },
            ref ty => ty.clone()
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
        use std::ops::Deref;
        match *self.1.borrow() {
            TyVarSubst::Bound{..} => write!(f, "'a{} b{:p}", self.0, self.1.deref()),
            TyVarSubst::Unbound{..} => write!(f, "'a{} u{:p}", self.0, self.1.deref())
        }
    }
}

impl fmt::Debug for TyCon {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::TyCon::*;
        let v = match *self {
            I32   => "i32",
            Bool  => "bool",
            Unit  => "()",
            Func  => "->",
            Cus(ref nm) => nm.as_str(),
        };
        write!(f, "{}", v)
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
            bound_tv.insert(v.clone());
        }
        let ftv = self.ty.free_tyvars();
        ftv.difference(&bound_tv);
        ftv
    }
    pub fn instantiate(&self, level: u32) -> (Vec<TyVar>, Type) {
        let mut subst = Subst::new();
        let mut tvs   = Vec::new();
        for bv in &self.bound_vars {
            let tv = fresh_tyvar(level);
            tvs.push(tv.clone());
            subst.bind(bv.clone(), Type::Var(tv));
        }
        (tvs, subst.apply(self.ty()))
    }
    pub fn apply_subst(&mut self) {
        self.ty = self.ty().apply_subst();
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
    
