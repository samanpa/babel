use std::rc::Rc;
use super::subst::Subst;
use std::collections::HashSet;

#[derive(Debug,Copy,Clone,Hash,Eq,PartialEq)]
pub struct TyVar(u32);

pub fn fresh_tyvar() -> TyVar {
    TyVar(::fresh_id())
}

pub fn mk_tyvar(id: u32) -> TyVar {
    TyVar(id)
}

#[derive(Debug,Clone,Hash,Eq,PartialEq)]
pub enum Type {
    Con(Rc<String>),
    App(Box<Type>, Vec<Type>),
    Var(TyVar)
}

#[derive(Debug,Clone,Hash,Eq,PartialEq)]
pub struct ForAll {
    bound_vars: Vec<TyVar>,
    ty: Type
}


impl Type {
    pub fn is_monotype(&self) -> bool {
        use self::Type::*;
        match *self {
            Con(_) => true,
            Var(_) => false,
            App(ref ty, ref args)  => {
                args.iter()
                    .fold( ty.is_monotype(),
                           | prev, ty | prev && ty.is_monotype())
            }
        }
    }

    pub fn free_tyvars(&self) -> HashSet<TyVar>
    {
        use self::Type::*;
        let mut res = HashSet::new();
        match *self {
            Con(_) => (),
            Var(v) => {res.insert(v);}
            App(ref ty, ref args) => {
                res = args.iter()
                    .fold( ty.free_tyvars(),
                           | mut ftv, ty | {
                               ftv = &ftv | & ty.free_tyvars();
                               ftv
                           });
            }
        }
        res
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
    /*
    pub fn mk_subst(&self, monotypes: &Vec<Type<u32>>) -> super::subst::Subst {
        let mut subst = super::subst::Subst::new();
        for (bound_var, monoty) in self.bound_vars.iter().zip(monotypes) {
            subst.bind(*bound_var, (*monoty).clone());
        }
        subst
    }
*/

}

pub (super) fn generalize(ty: Type, env: &super::env::Env) -> ForAll {
    let ftv1 = ty.free_tyvars();
    let ftv2 = env.free_tyvars();
    let ftv  = ftv1.difference(&ftv2)
        .cloned()
        .collect();
    ForAll::new(ftv, ty)
}
    
