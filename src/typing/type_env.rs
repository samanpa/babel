use union_find::{NodeId, DisjointSet};
use super::types::{Type, TyVar};
use ::Error;

#[derive(Debug)]
pub (super) struct TypeEnv {
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
                let ty = self.set.find(NodeId(*id as usize)).clone();
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
                if *l == *r && lk == rk {
                } else {
                    let msg = format!(
                        "Can not unify {:?} with {:?}",
                        lhs,
                        rhs
                    );
                    return Err(Error::new(msg));
                }
            }
            (&App(ref lty, ref largs), &App(ref rty, ref rargs)) => {
                if largs.len() != rargs.len() {
                    return cannot_unify(lhs, rhs);
                }
                else {
                    self.unify(lty, rty)?;
                    for (larg, rarg) in largs.iter().zip(rargs) {
                        self.unify(larg, rarg)?;
                    }
                }
            }
            (&Var(tyvar1), &Var(tyvar2)) => {
                self.set.merge(
                    NodeId(tyvar1.0 as usize),
                    NodeId(tyvar2.0 as usize)
                );
            }
            (&Var(tyvar), ty) |
            (ty, &Var(tyvar)) => {
                if occurs(tyvar, ty) {
                    return cannot_unify(&Type::Var(tyvar), ty);
                }
                let old = {
                    let new_ty = self.set.find(NodeId(tyvar.0 as usize));
                    let old = new_ty.clone();
                    *new_ty = ty.clone();
                    old
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

