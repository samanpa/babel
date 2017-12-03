use std::collections::HashMap;
use super::types::{Type,Function};

#[derive(Hash,Eq,PartialEq,Debug)]
pub enum SubType {
    Positional(u32),
    Explicit(u32),
}

#[derive(Debug)]
pub struct Subst {
    map: HashMap<SubType,Type<u32>>
}

impl Subst {
    pub fn new() -> Self {
        Subst{ map: HashMap::new() }
    }

    pub fn bind(&mut self, var: SubType, ty: Type<u32>) {
        self.map.insert(var, ty);
    }
    
    pub fn subst(&mut self, ty: &Type<u32>) -> Type<u32> {
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
                match self.map.get(&SubType::Explicit(id)) {
                    Some(ref ty) => (*ty).clone(),
                    None         => TyVar(id),
                }
            }
        }
    }

    /*
    pub fn unify(&mut self, lhs: &Type<u32>, rhs: &Type<u32>) -> Result<bool> {
        use types::Type::*;
        let res = match (lhs, rhs) {
            (&Bool, &Bool) => true,
            (&I32,  &I32)  => true,
            (&Unit, &Unit) => true,
            (&TyVar(t), _) => {self.bind(true,
            (_, &TyVar(_)) => true,
            (&Func(ref lhs), &Func(ref rhs)) => {
                lhs.return_ty == rhs.return_ty &&
                    lhs.params_ty().len() == rhs.params_ty().len() &&
                    lhs.params_ty().iter().zip(rhs.params_ty())
                       .fold(true, |prev, (lty, rty)| prev && (lty == rty))
            },
            _ => false,
        };       
        Ok<res>
    }
     */
}
