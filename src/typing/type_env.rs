use union_find::DisjointSet;
use super::types::{Type, TyVar};

#[derive(Debug)]
pub struct TypeEnv {
    set: DisjointSet<Type>
}


impl TypeEnv {
    pub fn new() -> Self {
        let set = DisjointSet::new();
        TypeEnv{ set }
    }

    pub fn fresh_tyvar(&mut self) -> TyVar {
        let tyid  = self.set.next_node_id().0 as u32;
        let tyvar = TyVar(tyid);
        self.set.add(Type::Var(tyvar));
        tyvar
    }
}

