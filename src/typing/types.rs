use std::rc::Rc;
use super::subst::Subst;
use std::collections::HashSet;

#[derive(Debug,Clone,Hash,Eq,PartialEq)]
pub enum Type {
    TyCon(Rc<String>),
    TyApp(Box<Type>, Vec<Type>),
    TyVar(u32)
}

#[derive(Debug,Clone,Hash,Eq,PartialEq)]
pub struct ForAll {
    bound_vars: Vec<u32>,
    ty: Type
}


impl Type {
    pub fn is_monotype(&self) -> bool {
        use self::Type::*;
        match *self {
            TyCon(_) => true,
            TyVar(_) => false,
            TyApp(ref ty, ref args)  => {
                args.iter()
                    .fold( ty.is_monotype(),
                           | prev, ty | prev && ty.is_monotype())
            }
        }
    }

    pub fn free_tyvars(&self) -> HashSet<u32>
    {
        use self::Type::*;
        let mut res = HashSet::new();
        match *self {
            TyCon(_) => (),
            TyVar(v) => {res.insert(v);}
            TyApp(ref ty, ref args) => {
                res = args.iter()
                    .fold( ty.free_tyvars(),
                           | mut ftv, ty | {
                               ftv.union(& ty.free_tyvars());
                               ftv
                           });
            }
        }
        res
    }
}

impl ForAll {
    pub fn new(bound_vars: Vec<u32>, ty: Type) -> Self {
        ForAll{ bound_vars, ty }
    }
    pub fn bound_vars(&self) -> &Vec<u32> {
        &self.bound_vars
    }
    pub fn ty(&self) -> &Type {
        &self.ty
    }
    pub fn is_monotype(&self) -> bool {
        self.bound_vars.len() == 0
    }
    pub fn free_tyvars(&self) -> HashSet<u32>
    {
        let mut bound_tv = HashSet::new();
        for v in self.bound_vars() {
            bound_tv.insert(*v);
        }
        let ftv = self.ty.free_tyvars();
        ftv.difference(&bound_tv);
        ftv
    }
    pub fn instantiate(&self) -> Type {
        let mut subst = Subst::new();
        for bv in &self.bound_vars {
            subst.bind(*bv, Type::TyVar(::fresh_id()));
        }
        subst.apply(self.ty())
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
    ftv1.difference(&ftv2);
    let ftv  = ftv1.iter()
        .map( |ftv| *ftv)
        .collect();
    ForAll::new(ftv, ty)
}
    
/*
//We should be implemententing a unifiable trait
pub fn unifiable(lhs: &Type<u32>, rhs: &Type<u32>) -> bool {
    let res = super::unify(lhs, rhs);
    match res {
        Err(e) => { println!("{}", e); false }
        Ok(_)  => { true }
    }
}

#[derive(Debug,Clone,Hash,Eq,PartialEq)]
pub struct Function<T> {
    params_ty: Vec<Type<T>>,
    return_ty: Type<T>
}

impl <T> Function<T> {
    pub fn new(params_ty: Vec<Type<T>>, return_ty: Type<T>) -> Self {
        Self{ params_ty, return_ty }
    }
    
    pub fn params_ty(&self) -> &Vec<Type<T>> {
        &self.params_ty
    }

    pub fn return_ty(&self) -> &Type<T> {
        &self.return_ty
    }
}
*/
