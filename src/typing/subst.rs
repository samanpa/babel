use std::collections::HashMap;
use super::types::Type;
use ::Result;

#[derive(Debug)]
pub struct Subst {
    map: HashMap<u32,Type>,
}

impl Subst {
    pub fn new() -> Self {
        Subst{ map: HashMap::new() }
    }

    pub fn bind(&mut self, tyvar: u32, ty: Type) {
        self.map.insert(tyvar, ty);
    }

    pub fn compose(self, rhs: &Subst) -> Result<Subst> {
        let mut subst = self;
        for (tyvar, ty) in &rhs.map {
            let ty = subst.apply(ty);
            subst.map.insert(*tyvar, ty);
        }
        Ok(subst)
    }
    
    pub fn apply(&self, ty: &Type) -> Type {
        use types::Type::*;
        match *ty {
            TyCon(ref con) => TyCon(con.clone()),
            TyApp(ref ty, ref args) => {
                let ty = self.apply(ty);
                let args = args.iter()
                    .map(|ty| self.apply(ty))
                    .collect();
                TyApp(Box::new(ty), args)
            }
            TyVar(id) => {
                match self.map.get(&id) {
                    Some(ref ty) => (*ty).clone(),
                    None         => TyVar(id),
                }
            }
        }
    }
}
