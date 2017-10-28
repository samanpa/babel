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

pub type VarRef = ::rename::Var;

pub enum Literal {
    Unit,
    I32(i32),
    Bool(bool),
}

pub struct Lambda {
    name: VarRef,
    params: Vec<VarRef>,
    body: Box<Expr>,
}

pub enum Expr {
    //Convert these into Type constants ?
    UnitLit,
    I32Lit(i32),
    BoolLit(bool),

    Lambda(Lambda),
    App{callee: Box<Expr>, args: Vec<Expr> },
    Var(VarRef),
    If{cond: Box<Expr>, texpr: Box<Expr>, fexpr: Box<Expr> }
}

impl Module {
    pub fn new(name: String) -> Self {
        Self{name, types: vec![], funcs: vec![]}
    }

    pub fn add_func(&mut self, lam: Lambda) {
        self.funcs.push(lam)
    }

    pub fn add_type(&mut self, ty: Type) {
        self.types.push(ty)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
