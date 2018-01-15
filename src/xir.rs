//explicitly typed IR.
//   Adds type abstractions and type applications as values as described by
//   "On The Type Structure of Standard ML" Robert Harper.

use std::rc::Rc;
use std::fmt;
use ::types::{Type,TyVar};

#[derive(Debug)]
pub struct Module {
    name:  String,
    decls: Vec<Decl>,
}

#[derive(Debug,Clone)]
pub struct FnProto {
    params: Vec<Ident>
}

#[derive(Debug)]
pub enum Decl {
    Extern(Ident, Type),
    Func(Ident, Rc<Lam>),
}

#[derive(Clone)]
pub struct Ident {
    name: Rc<String>,
    id: u32,
    ty: Type,
}

#[derive(Debug)]
pub struct Lam {
    proto: FnProto,
    body:  Expr,
}

pub struct Let {
    id:   Ident,
    bind: Expr,
    expr: Expr,
}

#[derive(Debug)]
pub struct If {
    cond:  Expr,
    texpr: Expr,
    fexpr: Expr
}

#[derive(Debug)]
pub enum Expr {
    Lam(Rc<Lam>),
    App(Box<Expr>, Box<Expr>),
    UnitLit,
    I32Lit(i32),
    BoolLit(bool),
    Var(Ident),
    If(Box<If>),
    Let(Box<Let>),
    TyLam(Vec<TyVar>, Box<Expr>),
    TyApp(Box<Expr>, Vec<Type>),
}

impl Module {
    pub fn new(name: String, decls: Vec<Decl>) -> Self {
        Self{name, decls}
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn decls(&self) -> &Vec<Decl> {
        &self.decls
    }

    pub fn add_decl(&mut self, decl: Decl) {
        self.decls.push(decl)
    }
}

impl Ident {
    pub fn new(name: Rc<String>, ty: Type, id: u32) -> Self {
        Ident{name, ty, id}
    }
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

impl fmt::Debug for Ident {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{} :: {:?}", self.name, self.id, self.ty)
    }
}

impl FnProto {
    pub fn new(params: Vec<Ident>) -> Self {
        FnProto{ params }
    }
    pub fn params(&self) -> &Vec<Ident> {
        &self.params
    }
}

impl Lam {
    pub fn new(proto: FnProto, body: Expr) -> Self {
        Lam{proto, body}
    }
    pub fn proto(&self) -> &FnProto {
        &self.proto
    }
//    pub fn take_proto(self) -> FnProto {
//        self.proto
//    }
    pub fn body(&self) -> &Expr {
        &self.body
    }
}

impl If {
    pub fn new(cond: Expr, texpr: Expr, fexpr: Expr) -> Self {
        If{cond, texpr, fexpr}
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
}

impl Let {
    pub fn new(id: Ident, bind: Expr, expr: Expr) -> Self {
        Let{id, bind, expr}
    }
    pub fn id(&self) -> &Ident {
        &self.id
    }
    pub fn bind(&self) -> &Expr {
        &self.bind
    }
    pub fn expr(&self) -> &Expr {
        &self.expr
    }
}

impl fmt::Debug for Let {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "let {:?} = {:?}\n{:?}", self.id, self.bind, self.expr)
    }
}

