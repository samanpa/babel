#[derive(Debug)]
pub enum Type {
    TyApp(String, Vec<Type>),
    TVar(String)
}

#[derive(Debug)]
pub struct Module {
    decls: Vec<Decl>,
}

#[derive(Debug)]
pub enum Decl {
    Extern(String, Type),
    Func(String, Lam),
}

#[derive(Debug)]
pub struct Lam {
    param: String,
    body:  Expr
}

#[derive(Debug)]
pub struct If {
    cond:  Expr,
    texpr: Expr,
    fexpr: Expr,
}

#[derive(Debug)]
pub enum Expr {
    Lam(Box<Lam>),
    App(Box<Expr>, Box<Expr>),
    UnitLit,
    I32Lit(i32),
    BoolLit(bool),
    Var(String, Vec<Type>),
    If(Box<If>),
    Let(String,Box<Expr>, Box<Expr>)
}

impl Module {
    pub fn new(decls: Vec<Decl>) -> Self {
        Self{decls}
    }

    pub fn decls(&self) -> &Vec<Decl> {
        &self.decls
    }
}

fn make_func(param: Type, ret: Type) -> Type {
    Type::TyApp("->".to_string(), vec![param, ret])
}

impl Lam {
    pub fn new(param: String, body: Expr) -> Self {
        Lam{param, body}
    }
    pub fn body(&self) -> &Expr {
        &self.body
    }
    pub fn param(&self) -> &String {
        &self.param
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
