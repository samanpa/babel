use std::collections::HashMap;
use super::types::{Type,Function};

#[derive(Debug)]
pub struct Subst {
    map: HashMap<u32,Type<u32>>,
    range: Vec<Type<u32>>
}

impl Subst {
    pub fn new() -> Self {
        Subst{ map: HashMap::new(), range: vec![] }
    }

    pub fn bind(&mut self, tyvar: u32, ty: Type<u32>) {
        self.range.push(ty.clone());
        self.map.insert(tyvar, ty);
    }
    
    pub fn subst(&self, ty: &Type<u32>) -> Type<u32> {
        use types::Type::*;
        match *ty {
            Bool => Bool,
            I32  => I32,
            Unit => Unit,
            TyCon(_) => unimplemented!(),
            Func(ref func_ty) => {
                let return_ty = self.subst(func_ty.return_ty());
                let params_ty = func_ty.params_ty().iter()
                    .map( |pty| self.subst(pty))
                    .collect();
                let func = Function::new(params_ty, return_ty);
                Func(Box::new(func))
            }
            TyVar(id) => {
                match self.map.get(&id) {
                    Some(ref ty) => (*ty).clone(),
                    None         => TyVar(id),
                }
            }
        }
    }

    pub fn range(&self) -> &Vec<Type<u32>> {
        &self.range
    }
}
