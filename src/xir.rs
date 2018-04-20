//explicitly typed IR.
//   Adds type abstractions and type applications as values as described by
//   "On The Type Structure of Standard ML" Robert Harper.
//System F like.

use std::rc::Rc;
use std::fmt;
use ::types::{Type,TyVar};

#[derive(Debug)]
pub struct Module {
    name:  String,
    decls: Vec<Decl>,
}

#[derive(Debug)]
pub enum Decl {
    Extern(Symbol),
    Let(Bind),
}

#[derive(Clone,Hash,Eq,PartialEq)]
pub struct Symbol {
    name: Rc<String>,
    id: u32,
    ty: Type
}

pub enum Bind {
    NonRec{symbol: Symbol, expr: Expr},
    //Rec(Vec<(String,Expr)>)
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
    App(u32, Box<Expr>, Box<Expr>),
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
    pub fn non_rec(symbol: Symbol, expr: Expr) -> Self {
        Bind::NonRec{symbol, expr}
    }
}

impl fmt::Debug for Bind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Bind::*;
        match *self {
            NonRec{ref symbol, ref expr} => {
                write!(f, "{:?} = {:?}", symbol, expr)
            }
        }
    }
}

impl fmt::Debug for Let {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "let {:?}\n{:?}", self.bind, self.expr)
    }
}

