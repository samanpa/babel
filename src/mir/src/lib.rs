#[repr(i8)]
#[derive(PartialEq,Eq,Clone,Copy)]
pub enum BaseType {
    Unit,
    Bool,
    Int32
}

pub enum Type {
    BaseType(BaseType),
    FunctionType{ params_ty: Vec<Type>, return_ty: Vec<Type> }
}

pub struct Variable(String);
pub struct VarRef(i32);
pub struct TypeRef(i32);

pub struct Param {
    name: VarRef,
    ty: Type,
}

pub enum Literal {
    Unit,
    Int32(i32),
    Bool(bool),
}

pub enum Expr {
    Lambda{name: VarRef, params: Vec<Param>, body: Box<Expr>},
    Application{name: VarRef, args: Box<Expr> },
    Literal{val: Literal },
    Variable{name: VarRef },
    IfExpr{cond: Box<Expr>, true_expr: Box<Expr>, false_expr: Option<Box<Expr>> }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
