use super::unify::UnificationTable;
use super::{ForAll, Type};
use crate::idtree::Symbol;
use crate::scoped_map::ScopedMap;
use crate::types::{Subst, TyVar};
use crate::{Error, Result};

#[derive(Debug)]
pub(super) struct Env {
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

    pub fn lookup(&self, id: &Symbol) -> Result<ForAll> {
        match self.map.get(&id.id()) {
            Some(ty) => Ok(ty.clone()),
            None => Err(Error::new(format!("Could not find {:?}", id))),
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

    pub fn unify<'a>(&mut self, lhs: &'a Type, rhs: &'a Type) -> crate::Result<()> {
        self.unify_table.unify(lhs, rhs)
    }

    pub fn fresh_tyvar(&mut self, level: u32) -> TyVar {
        let tyvar = TyVar::fresh(level);
        self.unify_table.add(tyvar.clone());
        tyvar
    }

    pub(super) fn instantiate(&mut self, scheme: &ForAll, level: u32) -> (Vec<TyVar>, Type) {
        //FIXME: does this even make sense
        let mut subst = Subst::new();
        let mut tvs = Vec::new();
        for bv in scheme.bound_vars() {
            let tv = self.fresh_tyvar(level);
            tvs.push(tv.clone());
            subst.bind(bv, Type::Var(tv));
        }
        (tvs, subst.apply(scheme.ty()))
    }
}
