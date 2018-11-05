use ::{Result,Error};
use idtree::Symbol;
use scoped_map::ScopedMap;
use super::types::{TyVar,ForAll, Type};
use super::unify::UnificationTable;

#[derive(Debug)]
pub (super) struct Env {
    map: ScopedMap<u32, ForAll>,
    unify_table: UnificationTable,
}

impl Env {
    pub fn new() -> Self {
        Self {
            map: ScopedMap::new(),
            unify_table: UnificationTable::new(),
        }
    }

    pub fn reset(&mut self) {
        self.unify_table.reset();
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

    pub fn begin_scope(&mut self) {
        self.map.begin_scope()
    }

    pub fn end_scope(&mut self) {
        self.map.end_scope()
    }

    pub fn apply(&mut self, ty: &Type) -> Type {
        self.unify_table.apply_subst(ty)
    }

    pub fn unify<'a>(&mut self, lhs: &'a Type, rhs: &'a Type) -> ::Result<()> {
        self.unify_table.unify(lhs, rhs)
    }

    pub fn fresh_tyvar(&mut self, level: u32) -> TyVar {
        let tyvar = super::types::fresh_tyvar(level);
        self.unify_table.add(tyvar.clone());
        tyvar
    }
}
