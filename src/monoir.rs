use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Unit,
    Bool,
    I32,
    Function {
        params_ty: Vec<Type>,
        return_ty: Box<Type>,
    },
}

#[derive(Debug)]
pub struct Module {
    pub name: String,
    pub types: Vec<Type>,
    pub funcs: Vec<Bind>,
    pub ext_funcs: Vec<Symbol>,
}

#[derive(Debug)]
pub struct Symbol {
    pub name: Rc<String>,
    pub ty: Type,
    pub id: u32,
}

#[derive(Debug)]
pub enum Literal {
    Unit,
    I32(i32),
    Bool(bool),
}

#[derive(Debug)]
pub struct Bind {
    pub sym: Symbol,
    pub expr: Expr,
}

#[derive(Debug)]
pub struct Lam {
    pub params: Vec<Symbol>,
    pub body: Expr,
}

#[derive(Debug)]
pub enum Expr {
    UnitLit,
    I32Lit(i32),
    BoolLit(bool),
    Lam(Box<Lam>),
    App(Box<Expr>, Vec<Expr>),
    Var(Symbol),
    //FIXME: introduce an If struct to reduce number or allocations
    If(Box<Expr>, Box<Expr>, Box<Expr>, Type),
    //FIXME: introduce an Let struct to reduce number or allocations
    Let(Box<Bind>, Box<Expr>),
}

impl Module {
    pub fn new(name: String) -> Self {
        Self {
            name,
            types: vec![],
            funcs: vec![],
            ext_funcs: vec![],
        }
    }
}

impl Bind {
    pub fn new(sym: Symbol, expr: Expr) -> Self {
        Bind { sym, expr }
    }
}

impl Lam {
    pub fn new(params: Vec<Symbol>, body: Expr) -> Self {
        Lam { params, body }
    }
}

impl Symbol {
    pub fn new(name: Rc<String>, ty: Type, id: u32) -> Self {
        Self { name, ty, id }
    }
    pub fn id(&self) -> u32 {
        self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn ty(&self) -> &Type {
        &self.ty
    }
}
