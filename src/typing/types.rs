#[derive(Debug,Clone,Hash,Eq,PartialEq)]
pub enum Type<T> {
    Bool,
    I32,
    Unit,
    TyCon(String),
    Func(Box<Function<T>>),
    TyVar(T)
}

#[derive(Debug,Clone,Hash,Eq,PartialEq)]
pub struct ForAll<T> {
    bound_vars: Vec<T>,
    ty: Type<T>
}

impl <T> ForAll<T> {
    pub fn new(bound_vars: Vec<T>, ty: Type<T>) -> Self {
        ForAll{ bound_vars, ty }
    }
    pub fn bound_vars(&self) -> &Vec<T> {
        &self.bound_vars
    }
    pub fn ty(&self) -> &Type<T> {
        &self.ty
    }
    pub fn is_monotype(&self) -> bool {
        self.bound_vars.len() == 0
    }
}
impl ForAll<u32> {
    pub fn mk_subst(&self, monotypes: &Vec<Type<u32>>) -> super::subst::Subst {
        let mut subst = super::subst::Subst::new();
        for (bound_var, monoty) in self.bound_vars.iter().zip(monotypes) {
            subst.bind(*bound_var, (*monoty).clone());
        }
        subst
    }
    
}

impl <T> Type<T> {
    pub fn is_monotype(&self) -> bool {
        use self::Type::*;
        match *self {
            Bool |
            I32  |
            Unit => true,
            TyCon(_)     => true, //FIXME
            Func(ref f)  => f.is_monotype(), //FIXME
            TyVar(_)     => false,
        }
    }
}

//We should be implemententing a unifiable trait
pub fn unifiable<T>(lhs: &Type<T>, rhs: &Type<T>) -> bool {
    use types::Type::*;
    match (lhs, rhs) {
        (&Bool, &Bool) => true,
        (&I32,  &I32)  => true,
        (&Unit, &Unit) => true,
        (&TyVar(_), _) => true,
        (_, &TyVar(_)) => true,
        (&TyCon(ref a), &TyCon(ref b)) => a == b,
        (&Func(ref lhs), &Func(ref rhs)) => {
            unifiable(&lhs.return_ty, &rhs.return_ty) &&
                lhs.params_ty().len() == rhs.params_ty().len() &&
                lhs.params_ty().iter().zip(rhs.params_ty())
                .fold(true, |prev, (lty, rty)| prev && unifiable(lty, rty))
        },
        _ => false,
    }
}

#[derive(Debug,Clone,Hash,Eq,PartialEq)]
pub struct Function<T> {
    params_ty: Vec<Type<T>>,
    return_ty: Type<T>
}

impl <T> Function<T> {
    pub fn new(params_ty: Vec<Type<T>>, return_ty: Type<T>) -> Self {
        Self{ params_ty, return_ty }
    }
    
    pub fn params_ty(&self) -> &Vec<Type<T>> {
        &self.params_ty
    }

    pub fn return_ty(&self) -> &Type<T> {
        &self.return_ty
    }

    pub fn is_monotype(&self) -> bool {
        self.params_ty.iter()
            .fold( self.return_ty.is_monotype(),
                   | prev, ty | prev && ty.is_monotype())
    }
}
