#[derive(Debug,Clone,Hash,Eq,PartialEq)]
pub enum Type<T> {
    Bool,
    I32,
    Unit,
    TyCon(String),
    Func(Box<Function<T>>),
    TyVar(T, TyVarFlavour)
}

#[derive(Debug,Clone,Hash,Eq,PartialEq,Copy)]
pub enum TyVarFlavour {
    Bound,
    Free,
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
            Bool     |
            I32      |
            TyCon(_) | // FIXME: What about list<K>
            Unit         => true,
            TyVar(_, _)  => false,
            Func(ref f)  => {
                f.params_ty
                    .iter()
                    .fold( f.return_ty.is_monotype(),
                           | prev, ty | prev && ty.is_monotype())
            }
        }
    }

    pub fn free_tyvars(&self) -> Vec<T>
        where T: Clone
    {
        use self::Type::*;
        use self::TyVarFlavour::*;
        match *self {
            Bool     |
            I32      |
            Unit     |
            TyCon(_) | //FIXME
            TyVar(_, Bound) => vec![],
            TyVar(ref k, Free)  => vec![k.clone()],
            Func(ref f)  => {
                f.params_ty
                    .iter()
                    .fold( f.return_ty.free_tyvars(),
                           | mut ftv, ty | {
                               ftv.append(&mut ty.free_tyvars());
                               ftv
                           })
            }
        }
    }
}

//We should be implemententing a unifiable trait
pub fn unifiable(lhs: &Type<u32>, rhs: &Type<u32>) -> bool {
    let res = super::unify(lhs, rhs);
    match res {
        Err(e) => { println!("{}", e); false }
        Ok(_)  => { true }
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
}
