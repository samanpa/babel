#![allow(dead_code)]

#[derive(Debug)]
pub struct TopLevel {
    decls: Vec<TopDecl>
}

#[derive(Debug)]
pub enum TopDecl {
    Extern{name: String, ty: Type },
    Use{name: String},
    Lam(Box<Lam>),
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
pub struct Lam {
    name:      String,
    params:    Vec<Param>,
    return_ty: Type, 
    body:      Expr
}

#[derive(Debug)]
pub struct If {
    cond: Expr,
    true_expr: Expr,
    false_expr: Expr,
}

#[derive(Debug)]
pub enum Expr {
    Lam(Box<Lam>),
    App{callee: Box<Expr>, args: Vec<Expr> },
    UnitLit,
    I32Lit(i32),
    BoolLit(bool),
    Var{name: String },
    If(Box<If> )
}

impl TopLevel {
    pub fn new(decls: Vec<TopDecl>) -> Self {
        Self{decls}
    }

    pub fn decls(&self) -> &Vec<TopDecl> {
        &self.decls
    }
}

impl Lam {
    pub fn new(name: String, params: Vec<Param>, return_ty: Type
               , body: Expr) -> Self {
        Lam{name, params, return_ty, body}
    }
    pub fn body(&self) -> &Expr {
        &self.body
    }
}

impl Param {
    pub fn new(name: String, ty: Type) -> Self {
        Param{name, ty}
    }
}

impl If {
    pub fn new(cond: Expr, true_expr: Expr, false_expr: Expr) -> Self {
        If{cond, true_expr, false_expr}
    }
    pub fn cond(&self) -> &Expr {
        &self.cond
    }
    pub fn true_expr(&self) -> &Expr {
        &self.true_expr
    }
    pub fn false_expr(&self) -> &Expr {
        &self.false_expr
    }
}
