//explicitly typed IR.
//   Adds type abstractions and type applications as values as described by
//   "On The Type Structure of Standard ML" Robert Harper.
//System F like.

use std::rc::Rc;
use std::fmt;
use std::hash::{Hash, Hasher};
use types::{self, TyVar};

type Type = types::Type<TyVar>;

#[derive(Debug)]
pub struct Module {
    name:  String,
    decls: Vec<Decl>,
}

#[derive(Debug)]
pub enum Decl {
    Extern(Symbol),
    Let(Vec<Bind>),
}

#[derive(Clone,Eq,PartialEq)]
pub struct Symbol {
    name: Rc<String>,
    id: u32,
    ty: Type
}

pub struct Bind {
    symbol: Symbol,
    expr: Expr,
}

pub struct Let {
    bind: Bind,
    expr: Expr,
}

#[derive(Debug)]
pub struct If {
    cond:  Expr,
    texpr: Expr,
    fexpr: Expr,
    ty: Type
}

#[derive(Debug)]
pub enum Expr {
    UnitLit,
    I32Lit(i32),
    BoolLit(bool),
    Var(Symbol),
    If(Box<If>),
    Let(Box<Let>),
    Lam(Vec<Symbol>, Box<Expr>, Type),
    App(Box<Expr>, Vec<Expr>),
    TyLam(Vec<TyVar>, Box<Expr>),
    TyApp(Box<Expr>, Vec<Type>),
}

impl Hash for Symbol {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
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

    pub fn take_decls(self) -> Vec<Decl> {
        self.decls
    }

    pub fn add_decl(&mut self, decl: Decl) {
        self.decls.push(decl)
    }
}

impl Symbol {
    pub fn new(name: Rc<String>, ty: Type, id: u32) -> Self {
        Self{name, ty, id}
    }
    pub fn with_ty(&self, ty: Type) -> Self {
        Self::new(self.name.clone(), ty, self.id)
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

impl fmt::Debug for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}_{}: {:?}", self.name, self.id, self.ty)
    }
}

impl If {
    pub fn new(cond: Expr, texpr: Expr, fexpr: Expr, ty: Type) -> Self {
        If{cond, texpr, fexpr, ty}
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
    pub fn ty(&self) -> &Type {
        &self.ty
    }
}

impl Let {
    pub fn new(bind: Bind, expr: Expr) -> Self {
        Let{bind, expr}
    }
    pub fn bind(&self) -> &Bind {
        &self.bind
    }
    pub fn expr(&self) -> &Expr {
        &self.expr
    }
}

impl Bind {
    pub fn new(symbol: Symbol, expr: Expr) -> Self {
        Bind{symbol, expr}
    }

    pub fn symbol(&self) -> &Symbol {
        &self.symbol
    }

    pub fn expr(&self) -> &Expr {
        &self.expr
    }
}

impl fmt::Debug for Bind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} = {:#?}", self.symbol, self.expr)
    }
}

impl fmt::Debug for Let {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "let {:?}\n{:#?}", self.bind, self.expr)
    }
}

