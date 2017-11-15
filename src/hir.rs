use std::rc::Rc;

#[derive(Debug)]
pub struct TopLevel {
    decls: Vec<TopDecl>,
}

#[derive(Debug)]
pub enum TopDecl {
    Extern(Ident, Vec<Type>),
    Lam(Lam),
}

#[derive(Debug,Clone)]
pub struct Ident {
    name: Rc<String>,
    id: u32,
    ty: Type,
}

#[derive(Debug,Clone,PartialEq,Eq)]
pub enum Type {
    Bool,
    I32,
    Unit,
    Function{ params_ty: Vec<Type>, return_ty: Box<Type> },
    TyVar(u32),
}

#[derive(Debug)]
pub struct Lam {
    ident: Ident,
    params: Vec<Ident>,
    return_ty: Type,
    body:  Expr,
}

#[derive(Debug)]
pub struct If {
    cond:  Expr,
    texpr: Expr,
    fexpr: Expr,
    res_ty: Type,
}

#[derive(Debug)]
pub enum Expr {
    Lam(Box<Lam>),
    App{callee: Box<Expr>, args: Vec<Expr> },
    UnitLit,
    I32Lit(i32),
    BoolLit(bool),
    Var(Ident),
    If(Box<If>)
}

impl TopLevel {
    pub fn new(decls: Vec<TopDecl>) -> Self {
        Self{decls}
    }

    pub fn decls(&self) -> &Vec<TopDecl> {
        &self.decls
    }
    pub fn decls_mut(&mut self) -> &mut Vec<TopDecl> {
        &mut self.decls
    }
}

impl Ident {
    pub fn new(name: Rc<String>, ty: Type, id: u32) -> Self {
        Ident{name, ty, id}
    }

    //Rename
    pub fn name(&self) -> &Rc<String> {
        &self.name
    }
    pub fn ty(&self) -> &Type {
        &self.ty
    }
    pub fn id(&self) -> u32 {
        self.id
    }
}

impl Lam {
    pub fn new(ident: Ident, params: Vec<Ident>, body: Expr
               , return_ty: Type) -> Self {
        Lam{ident, params, body, return_ty}
    }
    pub fn body(&self) -> &Expr {
        &self.body
    }
    pub fn body_mut(&mut self) -> &mut Expr {
        &mut self.body
    }
    pub fn ident(&self) -> &Ident {
        &self.ident
    }
    pub fn return_ty(&self) -> &Type {
        &self.return_ty
    }
    pub fn params(&self) -> &Vec<Ident> {
        &self.params
    }
}

impl If {
    pub fn new(cond: Expr, texpr: Expr, fexpr: Expr, res_ty: Type) -> Self {
        If{cond, texpr, fexpr, res_ty}
    }
    pub fn cond(&self) -> &Expr {
        &self.cond
    }
    pub fn texpr(&self) -> &Expr {
        &self.texpr
    }
    pub fn fexpr(&self) -> &Expr {
        &self.fexpr
    }
    pub fn res_ty(&self) -> &Type {
        &self.res_ty
    }
}