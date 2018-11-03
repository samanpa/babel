use std::collections::HashMap;
use ::{Result,Error};
use idtree::Symbol;
use super::types::{TyVar,ForAll, Type};
use super::unify::UnificationTable;

use std::rc::Rc;
use std::cell::RefCell;

#[derive(Clone,Debug)]
pub (super) struct Env {
    map: HashMap<u32, ForAll>,
    pub (super) unify_table: Rc<RefCell<UnificationTable>>,
}

impl Env {
    pub fn new() -> Self {
        Self{
            map: HashMap::new(),
            unify_table: Rc::new(RefCell::new(UnificationTable::new())),
        }
    }

    pub fn reset(&mut self) {
        self.unify_table.borrow_mut().reset();
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
        self.unify_table.borrow_mut().apply_subst(ty)
    }

    pub fn unify<'a>(&mut self, lhs: &'a Type, rhs: &'a Type) -> ::Result<()> {
        self.unify_table.borrow_mut().unify(lhs, rhs)
    }

    pub fn fresh_tyvar(&mut self, level: u32) -> TyVar {
        let tyvar = super::types::fresh_tyvar(level);
        self.unify_table.borrow_mut().add(tyvar.clone());
        tyvar
    }
}
