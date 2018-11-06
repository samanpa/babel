use super::subst::Subst;
use super::{Type, TyVar};

#[derive(Debug,Clone,Hash,Eq,PartialEq)]
pub struct ForAll {
    bound_vars: Vec<TyVar>,
    ty: Type
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
}

