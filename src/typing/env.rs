use std::collections::{HashMap,HashSet};
use ::{Result,Error};
use idtree::Symbol;
use super::types::{TyVar,ForAll, Type};
use super::type_env::TypeEnv;

use std::rc::Rc;
use std::cell::RefCell;

#[derive(Clone,Debug)]
pub (super) struct Env {
    map: HashMap<u32, ForAll>,
    type_env: Rc<RefCell<TypeEnv>>,
}

impl Env {
    pub fn new() -> Self {
        Self{
            map: HashMap::new(),
            type_env: Rc::new(RefCell::new(TypeEnv::new())),
        }
    }
    
    pub fn lookup(&self, id: &Symbol) -> Result<ForAll> {
        match self.map.get(&id.id()) {
            Some(ty) => Ok(ty.clone()),
            None     => Err(Error::new(format!("Could not find {:?}", id)))
        }
    }

    pub fn extend(&mut self, id: &Symbol, ty: ForAll) {
        self.map.insert(id.id(), ty);
    }

    pub fn apply_subst(&self) -> Env {
        let env = self.clone();
        env
    }

    pub fn apply(&mut self, ty: &Type) -> Type {
        self.type_env.borrow_mut().apply_subst(ty)
    }


    pub fn unify<'a>(&mut self, lhs: &'a Type, rhs: &'a Type) -> ::Result<()>
    {
        self.type_env.borrow_mut().unify(lhs, rhs)
    }

    pub fn free_tyvars(&self) -> HashSet<TyVar> {
        let mut ftv = HashSet::new();
        for sigma in self.map.values() {
            ftv.extend(&sigma.free_tyvars());
        }
        ftv
    }

    pub fn fresh_tyvar(&mut self) -> TyVar {
        self.type_env.borrow_mut().fresh_tyvar()
    }
}
