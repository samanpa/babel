use std::rc::Rc;

#[derive(Debug,Clone)]
pub enum Type {
    Unit,
    Bool,
    I32,
    Function{ params_ty: Vec<Type>, return_ty: Box<Type> },
}

#[derive(Debug)]
pub struct Module {
    name:  String,
    types: Vec<Type>,
    funcs: Vec<Bind>,
    ext_funcs: Vec<Symbol>,
}

#[derive(Debug)]
pub struct Symbol {
    name: Rc<String>,
    ty:   Type,
    id:   u32,
}

#[derive(Debug)]
pub enum Literal {
    Unit,
    I32(i32),
    Bool(bool),
}

#[derive(Debug)]
pub struct Bind {
    sym: Symbol,
    expr: Expr,
}

#[derive(Debug)]
pub struct Lam {
    params: Vec<Symbol>,
    body: Expr,
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
    If(Box<Expr>, Box<Expr>, Box<Expr>, Type ),
    //FIXME: introduce an Let struct to reduce number or allocations
    Let(Box<Bind>, Box<Expr>),
}

impl Module {
    pub fn new(name: String) -> Self {
        Self{name, types: vec![], funcs: vec![], ext_funcs: vec![]}
    }

    pub fn name(&self) -> &String {
        &self.name
    }
    
    pub fn types(&self) -> &Vec<Type> {
        &self.types
    }
    
    pub fn funcs(&self) -> &Vec<Bind> {
        &self.funcs
    }

    pub fn externs(&self) -> &Vec<Symbol> {
        &self.ext_funcs
    }

    pub fn add_func(&mut self, lam: Bind) {
        self.funcs.push(lam)
    }

    pub fn add_type(&mut self, ty: Type) {
        self.types.push(ty)
    }

    pub fn add_extern(&mut self, name: Symbol) {
        self.ext_funcs.push(name)
    }
}

impl Bind {
    pub fn new(sym: Symbol, expr: Expr) -> Self {
        Bind{sym, expr}
    }
    pub fn symbol(&self) -> &Symbol {
        &self.sym
    }
    pub fn expr(&self) -> &Expr {
        &self.expr
    }
}

impl Lam {
    pub fn new(params: Vec<Symbol>, body: Expr) -> Self {
        Lam{params, body}
    }
    pub fn params(&self) -> &Vec<Symbol> {
        &self.params
    }
    pub fn body(&self) -> &Expr {
        &self.body
    }
}

impl Symbol {
    pub fn new(name: Rc<String>, ty: Type, id: u32) -> Self {
        Self{name, ty, id}
    }
    pub fn id(&self) -> u32 {
        self.id
    }
    pub fn name(&self) -> &String {
        &self.name
    }
    pub fn ty(&self) -> &Type {
        &self.ty
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
