use std::rc::Rc;
use std::fmt;
use ::types::Type;

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

pub struct Let {
    id:   Symbol,
    bind: Expr,
    expr: Expr,
}

pub struct Bind {
    symbol: Symbol, expr: Expr,
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
    Lam(Vec<Symbol>, Box<Expr>),
    App(u32, Box<Expr>, Box<Expr>),
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
        Symbol{name, ty, id}
    }
    pub fn with_ty(&self, ty: Type) -> Symbol {
        Symbol::new(self.name.clone(), ty, self.id)
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
    pub fn new(id: Symbol, bind: Expr, expr: Expr) -> Self {
        Let{id, bind: bind, expr}
    }
    pub fn id(&self) -> &Symbol {
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

impl Bind {
    pub fn new(symbol: Symbol, expr: Expr) -> Self {
        Self{symbol, expr}
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
        write!(f, "{:?} = {:?}", self.symbol, self.expr)
    }
}

