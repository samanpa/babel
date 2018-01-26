use std::rc::Rc;
use super::subst::Subst;
use std::collections::HashSet;
use std::fmt;

#[derive(Copy,Clone,Hash,Eq,PartialEq)]
pub struct TyVar(u32);

pub fn fresh_tyvar() -> TyVar {
    TyVar(::fresh_id())
}

pub fn mk_tyvar(id: u32) -> TyVar {
    TyVar(id)
}

#[derive(Clone,Hash,Eq,PartialEq)]
pub enum Type {
    Con(Rc<String>, u32),
    App(Box<Type>, Box<Type>),
    Var(TyVar)
}

#[derive(Clone)]
pub enum Kind {
    Type
}

#[derive(Debug,Clone,Hash,Eq,PartialEq)]
pub struct ForAll {
    bound_vars: Vec<TyVar>,
    ty: Type
}


impl Type {
    pub fn arity(&self) -> u32 {
        match *self {
            Type::App(ref l, _) => l.arity() - 1,
            Type::Con(_, arity) => arity,
            Type::Var(_)        => 0, //FIXME: not true when we have HKT
        }
    }

    pub fn free_tyvars(&self) -> HashSet<TyVar>
    {
        use self::Type::*;
        let mut res = HashSet::new();
        match *self {
            Con(_, _) => (),
            Var(v)    => {res.insert(v);}
            App(ref con, ref arg) => {
                let con_ftv = con.free_tyvars();
                let arg_ftv = arg.free_tyvars();
                res = &con_ftv | &arg_ftv;
            }
        }
        res
    }
}

impl fmt::Debug for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Type::*;
        match *self {
            Con(ref str, n)   => write!(f, "{}:{}", str, n),
            App(ref a, ref b) => write!(f, "App({:?}, {:?})", a, b),
            Var(v)            => write!(f, "{:?}", v),
        }
    }
}

impl fmt::Debug for TyVar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "'a{}", self.0)
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
    pub fn instantiate(&self) -> (Vec<TyVar>, Type) {
        let mut subst = Subst::new();
        let mut tvs   = Vec::new();
        for bv in &self.bound_vars {
            let tv = fresh_tyvar();
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
    
