use std::collections::HashMap;
use types::{TyVar, Type};

#[derive(Debug)]
pub struct Subst {
    map: HashMap<u32, Type<TyVar>>,
}

impl Default for Subst {
    fn default() -> Self {
        Self::new()
    }
}

impl Subst {
    pub fn new() -> Self {
        Subst {
            map: HashMap::new(),
        }
    }

    pub fn bind(&mut self, tyvar: &TyVar, ty: Type<TyVar>) {
        self.map.insert(tyvar.id, ty);
    }

    pub fn apply(&self, ty: &Type<TyVar>) -> Type<TyVar> {
        use self::Type::*;
        match *ty {
            Con(ref con, ref kind) => Con(con.clone(), kind.clone()),
            App(ref con, ref args) => {
                let con = self.apply(con);
                let args = args.iter().map(|arg| self.apply(arg)).collect();
                App(Box::new(con), args)
            }
            Var(ref tv) => match self.map.get(&tv.id) {
                Some(ty) => ty.clone(),
                None => Var(tv.clone()),
            },
        }
    }
}
