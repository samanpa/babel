#[derive(Debug,Clone,Hash)]
pub enum Type<T> {
    Bool,
    I32,
    Unit,
    TyCon(T),
    Function{ params_ty: Vec<Type<T>>, return_ty: Box<Type<T>> },
    TyVar(T),
}

impl <T> PartialEq for Type<T> {
    fn eq(&self, rhs: &Type<T>) -> bool {
        use types::Type::*;
        match (self, rhs) {
            (&Bool, &Bool) => true,
            (&I32,  &I32)  => true,
            (&Unit, &Unit) => true,
            (&TyVar(_), _) => true,
            (_, &TyVar(_)) => true,
            (&Function{params_ty: ref lparam, return_ty: ref lreturn},
             &Function{params_ty: ref rparam, return_ty: ref rreturn}) => {
                lreturn == rreturn &&
                    lparam.len() == rparam.len() &&
                    lparam.iter().zip(rparam)
                       .fold(true, |prev, (lty, rty)| prev && (lty == rty))
            },
            _ => false,
        }
    }
}
impl <T> Eq for Type<T> {}

