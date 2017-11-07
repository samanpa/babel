#![allow(dead_code)]

#[derive(Debug)]
pub struct TopLevel<Ident> {
    decls: Vec<TopDecl<Ident>>,
}

#[derive(Debug)]
pub struct FnProto<Ident> {
    name: Ident,
    params: Vec<Param<Ident>>,
    return_ty: Type
}

#[derive(Debug)]
pub enum TopDecl<Ident> {
    Extern(FnProto<Ident>),
    Use{name: String},
    Lam(Box<Lam<Ident>>),
}

#[derive(Debug,Clone,Eq,PartialEq)]
pub enum BaseType {
    Bool,
    I32,
    Unit,
}

#[derive(Debug,Clone,Eq,PartialEq)]
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
    proto: FnProto<Ident>,
    body:  Expr<Ident>
}

#[derive(Debug)]
pub struct If<Ident> {
    cond:  Expr<Ident>,
    texpr: Expr<Ident>,
    fexpr: Expr<Ident>,
    res_ty:Option<Type>, // The type of both texpr and fexpr
}

#[derive(Debug)]
pub enum Expr<Ident> {
    Lam(Box<Lam<Ident>>),
    App{callee: Box<Expr<Ident>>, args: Vec<Expr<Ident>> },
    UnitLit,
    I32Lit(i32),
    BoolLit(bool),
    Var(Ident),
    If(Box<If<Ident>>)
}

impl <Ident> TopLevel<Ident> {
    pub fn new(decls: Vec<TopDecl<Ident>>) -> Self {
        Self{decls}
    }

    pub fn decls(&self) -> &Vec<TopDecl<Ident>> {
        &self.decls
    }
}

impl <Ident> FnProto<Ident> {
    pub fn new(name: Ident, params: Vec<Param<Ident>>, return_ty: Type) -> Self {
        FnProto{name, params, return_ty}
    }
    //Rename
    pub fn name(&self) -> &Ident {
        &self.name
    }
    pub fn return_ty(&self) -> &Type {
        &self.return_ty
    }
    pub fn params(&self) -> &Vec<Param<Ident>> {
        &self.params
    }

    pub fn ty(&self) -> Type {
        let params_ty = self.params.iter()
            .map(|ref param| param.ty.clone())
            .collect();
        let return_ty = Box::new(self.return_ty.clone());
        Type::FunctionType{ params_ty, return_ty }
    }
}

impl <Ident> Lam<Ident> {
    pub fn new(name: Ident, params: Vec<Param<Ident>>, return_ty: Type
               , body: Expr<Ident>) -> Self {
        let proto = FnProto::new(name, params, return_ty);
        Lam{proto, body}
    }
    pub fn proto(&self) -> &FnProto<Ident> {
        &self.proto
    }
    pub fn body(&self) -> &Expr<Ident> {
        &self.body
    }
    pub fn name(&self) -> &Ident {
        &self.proto.name
    }
    pub fn return_ty(&self) -> &Type {
        &self.proto.return_ty
    }
    pub fn ty(&self) -> Type {
        self.proto.ty()
    }
    pub fn params(&self) -> &Vec<Param<Ident>> {
        &self.proto.params
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
    pub fn new(cond: Expr<Ident>, texpr: Expr<Ident>, fexpr: Expr<Ident>
               ,res_ty: Option<Type>) -> Self {
        If{cond, texpr, fexpr, res_ty}
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
    pub fn res_ty(&self) -> &Option<Type> {
        &self.res_ty
    }
}
