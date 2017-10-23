#![allow(dead_code)]

#[derive(Debug)]
pub struct TopLevel<Ident> {
    decls: Vec<TopDecl<Ident>>,
}

#[derive(Debug)]
pub enum TopDecl<Ident> {
    Extern{name: Ident, ty: Type},
    Use{name: String},
    Lam(Box<Lam<Ident>>),
}

#[derive(Debug,Clone)]
pub enum BaseType {
    Bool,
    I32,
    Unit,
}

#[derive(Debug,Clone)]
pub enum Type {
    BaseType(BaseType),
    FunctionType{ params_ty: Vec<Type>, return_ty: Box<Type> }
}

#[derive(Debug)]
pub struct Param<Ident> {
    name: Ident,
    ty: Type,
}

#[derive(Debug)]
pub struct Lam<Ident> {
    name:      Ident,
    params:    Vec<Param<Ident>>,
    return_ty: Type, 
    body:      Expr<Ident>
}

#[derive(Debug)]
pub struct If<Ident> {
    cond:  Expr<Ident>,
    texpr: Expr<Ident>,
    fexpr: Expr<Ident>,
}

#[derive(Debug)]
pub enum Expr<Ident> {
    Lam(Box<Lam<Ident>>),
    App{callee: Box<Expr<Ident>>, args: Vec<Expr<Ident>> },
    UnitLit,
    I32Lit(i32),
    BoolLit(bool),
    Var(Ident),
    If(Box<If<Ident>> )
}

impl <Ident> TopLevel<Ident> {
    pub fn new(decls: Vec<TopDecl<Ident>>) -> Self {
        Self{decls}
    }

    pub fn decls(&self) -> &Vec<TopDecl<Ident>> {
        &self.decls
    }
}

impl <Ident> Lam<Ident> {
    pub fn new(name: Ident, params: Vec<Param<Ident>>, return_ty: Type
               , body: Expr<Ident>) -> Self {
        Lam{name, params, return_ty, body}
    }
    pub fn name(&self) -> &Ident {
        &self.name
    }
    pub fn body(&self) -> &Expr<Ident> {
        &self.body
    }
    pub fn return_ty(&self) -> &Type {
        &self.return_ty
    }
    pub fn params(&self) -> &Vec<Param<Ident>> {
        &self.params
    }
}

impl <Ident> Param<Ident> {
    pub fn new(name: Ident, ty: Type) -> Self {
        Param{name, ty}
    }

    pub fn name(&self) -> &Ident {
        &self.name
    }

    pub fn ty(&self) -> &Type {
        &self.ty
    }
}

impl <Ident> If<Ident> {
    pub fn new(cond: Expr<Ident>, texpr: Expr<Ident>, fexpr: Expr<Ident>) -> Self {
        If{cond, texpr, fexpr}
    }
    pub fn cond(&self) -> &Expr<Ident> {
        &self.cond
    }
    pub fn texpr(&self) -> &Expr<Ident> {
        &self.texpr
    }
    pub fn fexpr(&self) -> &Expr<Ident> {
        &self.fexpr
    }
}
