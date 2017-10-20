#![allow(dead_code)]

#[derive(Debug)]
pub enum TopLevel {
    Extern{name: String, ty: Type },
    Use{name: String},
    Lambda(Box<Lambda>),
}

#[derive(Debug)]
pub enum BaseType {
    Bool,
    I32,
    Unit,
}

#[derive(Debug)]
pub enum Type {
    BaseType(BaseType),
    FunctionType{ params_ty: Vec<Type>, return_ty: Box<Type> }
}

#[derive(Debug)]
pub struct Param {
    name: String,
    ty: Type,
}

#[derive(Debug)]
pub enum Literal {
    Unit,
    I32(i32),
    Bool(bool),
}

#[derive(Debug)]
pub struct Lambda {
    name:      String,
    params:    Vec<Param>,
    return_ty: Type, 
    body:      Expr
}

#[derive(Debug)]
pub struct IfExpr {
    cond: Expr,
    true_expr: Expr,
    false_expr: Option<Expr>,
}

#[derive(Debug)]
pub enum Expr {
    Lambda(Box<Lambda>),
    Application{callee: Box<Expr>, args: Vec<Expr> },
    Literal(Literal),
    Variable{name: String },
    IfExpr( Box<IfExpr> )
}

impl Lambda {
    pub fn new(name: String, params: Vec<Param>, return_ty: Type
               , body: Expr) -> Self {
        Lambda{name, params, return_ty, body}
    }
}

impl Param {
    pub fn new(name: String, ty: Type) -> Self {
        Param{name, ty}
    }
}

impl IfExpr {
    pub fn new(cond: Expr, true_expr: Expr, false_expr: Option<Expr>) -> Self {
        IfExpr{cond, true_expr, false_expr}
    }
}
