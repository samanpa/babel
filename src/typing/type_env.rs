use union_find::{DisjointSet, self};
use super::types::{Type, TyVar};
use std::collections::HashMap;
use ::Error;
use std;

#[derive(Debug)]
pub (super) struct TypeEnv {
    subst: DisjointSet<u32, Type>,
    indices: HashMap<u32, u32>
}

impl union_find::Value for Type {}

impl TypeEnv {
    pub fn new() -> Self {
        let subst = DisjointSet::with_capacity(100);
        let indices = HashMap::new();
        TypeEnv{ subst, indices }
    }

    pub fn fresh_tyvar(&mut self) -> TyVar {
        let tyid  = self.indices.len() as u32;
        let tyvar = TyVar(tyid);
        let key   = self.subst.add(Type::Var(tyvar));
        self.indices.insert(tyid, key);
        tyvar
    }

    pub fn apply_subst(&mut self, ty: &Type) -> Type {
        use types::Type::*;
        let res = match ty {
            Con(ref con, ref kind) => Con(con.clone(), kind.clone()),
            App(ref con, ref args)  => {
                let con = self.apply_subst(con);
                let args = args.iter()
                    .map( |arg| self.apply_subst(arg))
                    .collect();
                App(Box::new(con), args)
            }
            Var(TyVar(id)) => {
                let key = *self.indices.get(id).unwrap();
                let ty = self.subst.find(key).clone();
                match ty {
                    Var(ty) => Var(ty),
                    ty      => self.apply_subst(&ty)
                }
            }
        };

        //println!("APPLY SUBST \n{:?} = {:?} \n{:#?}", ty, res, self);
        res
    }

    pub fn unify<'a>(&mut self, lhs: &'a Type, rhs: &'a Type) -> ::Result<()>
    {
        use types::Type::*;
        match (lhs, rhs) {
            (&Con(ref l, ref lk), &Con(ref r, ref rk)) => {
                if *l != *r || lk != rk {
                    return cannot_unify(lhs, rhs);
                }
            }
            (&App(ref lty, ref largs), &App(ref rty, ref rargs)) => {
                if largs.len() != rargs.len() {
                    return cannot_unify(lhs, rhs);
                }
                self.unify(lty, rty)?;
                for (larg, rarg) in largs.iter().zip(rargs) {
                    self.unify(larg, rarg)?;
                }
            }
            (&Var(tyvar1), &Var(tyvar2)) => {
                let key1 = *self.indices.get(&tyvar1.0).unwrap();
                let key2 = *self.indices.get(&tyvar2.0).unwrap();
                self.subst.merge(key1, key2);
            }
            (&Var(tyvar), ty) |
            (ty, &Var(tyvar)) => {
                if occurs(tyvar, ty) {
                    return cannot_unify(&Type::Var(tyvar), ty);
                }
                let old = {
                    let key = *self.indices.get(&tyvar.0).unwrap();
                    let new_ty = self.subst.find(key);
                    std::mem::replace(new_ty, ty.clone())
                };
                self.unify(&old, ty)?;
            }
            _ => {
                return cannot_unify(lhs, rhs);
            }
        };
        Ok(())
    }
}

fn cannot_unify( lhs: &Type, rhs: &Type) -> ::Result<()> {
    let msg = format!("Can not unify {:?} with {:?}", lhs, rhs);
    return Err(Error::new(msg));
}

fn occurs(tyvar: TyVar, ty: &Type) -> bool
{
    use types::Type::*;
    match *ty {
        Con(_, _) |
        Var(_)    => false,
        ref app @ App(_, _) => {
            app.free_tyvars()
                .iter()
                .fold( false, | acc, tv | acc || tyvar == *tv )
        }
    }
}

