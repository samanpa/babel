//Temporary name
pub struct Module {
    name:  String,
    types: Vec<Type>,
    funcs: Vec<Lambda>,
}

#[repr(i8)]
#[derive(PartialEq,Eq,Clone,Copy)]
pub enum BaseType {
    Unit,
    Bool,
    I32
}

pub enum Type {
    BaseType(BaseType),
    FunctionType{ params_ty: Vec<Type>, return_ty: Vec<Type> }
}

pub struct VarRef(i32);
pub struct TypeRef(i32);

pub struct Param {
    name: VarRef,
    ty: Type,
}

pub enum Literal {
    Unit,
    I32(i32),
    Bool(bool),
}

pub struct Lambda {
    name: VarRef,
    params: Vec<Param>,
    body: Box<Expr>,
}

pub enum Expr {
    //Convert these into applications?
    UnitLit,
    I32Lit(i32),
    BoolLit(bool),

    Lambda(Lambda),
    App{name: VarRef, args: Box<Expr> },
    Var(VarRef),
    If{cond: Box<Expr>, texpr: Box<Expr>, fexpr: Box<Expr> }
}

impl Module {
    pub fn new(name: String) -> Self {
        Self{name, types: vec![], funcs: vec![]}
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
