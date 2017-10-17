pub enum BaseType {
    Bool,
    I32,
    Unit,
}

pub enum Type {
    BaseType(BaseType),
    FunctionType{ params_ty: Vec<Type>, return_ty: Box<Type> }
}

pub struct Param {
    name: String,
    ty: Type,
}

pub enum Literal {
    Unit,
    I32(i32),
    Bool(bool),
}

pub struct Abstraction {
    name:      String,
    params:    Vec<Param>,
    return_ty: Type, 
    body:      Box<Expr>
}

pub struct IfExpr {
    cond: Box<Expr>,
    true_expr: Box<Expr>,
    false_expr: Option<Box<Expr>>
}

pub enum Expr {
    Abstraction(Abstraction),
    Application{name: String, args: Box<Expr> },
    Literal(Literal),
    Variable{name: String },
    IfExpr( IfExpr )
}

