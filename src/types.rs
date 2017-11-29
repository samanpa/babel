#[derive(Debug,Clone,Hash)]
pub enum Type<T> {
    Bool,
    I32,
    Unit,
    TyCon(T),
    Func(Box<Function<T>>),
    TyVar(T),
}

//We should be implemententing a unifiable trait
impl <T> PartialEq for Type<T> {
    fn eq(&self, rhs: &Type<T>) -> bool {
        use types::Type::*;
        match (self, rhs) {
            (&Bool, &Bool) => true,
            (&I32,  &I32)  => true,
            (&Unit, &Unit) => true,
            (&TyVar(_), _) => true,
            (_, &TyVar(_)) => true,
            (&Func(ref lhs), &Func(ref rhs)) => {
                lhs.return_ty == rhs.return_ty &&
                    lhs.params_ty().len() == rhs.params_ty().len() &&
                    lhs.params_ty().iter().zip(rhs.params_ty())
                       .fold(true, |prev, (lty, rty)| prev && (lty == rty))
            },
            _ => false,
        }
    }
}
impl <T> Eq for Type<T> {}


#[derive(Debug,Clone,Hash)]
pub struct Function<T> {
    params_ty: Vec<Type<T>>,
    return_ty: Type<T>,
    ty_vars:   Vec<Type<T>>
}

impl <T> Function<T> {
    pub fn new(ty_vars: Vec<Type<T>>, params_ty: Vec<Type<T>>
               , return_ty: Type<T>) -> Self {
        Self{ params_ty, return_ty, ty_vars }
    }
    
    pub fn params_ty(&self) -> &Vec<Type<T>> {
        &self.params_ty
    }

    pub fn return_ty(&self) -> &Type<T> {
        &self.return_ty
    }
    
    pub fn ty_vars(&self) -> &Vec<Type<T>> {
        &self.ty_vars
    }
}
