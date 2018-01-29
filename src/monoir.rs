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
    funcs: Vec<Func>,
    ext_func: Vec<FnProto>,
}

#[derive(Debug)]
pub struct TermVar {
    name: Rc<String>,
    ty:   Type,
    id:   u32,
}

#[derive(Debug)]
pub struct FnProto {
    ident: TermVar,
    params: Vec<TermVar>
}

#[derive(Debug)]
pub enum Literal {
    Unit,
    I32(i32),
    Bool(bool),
}

#[derive(Debug)]
pub struct Func {
    name: TermVar,
    body: Expr,
}

#[derive(Debug)]
pub struct Lam {
    params: Vec<TermVar>,
    body: Expr,
}

#[derive(Debug)]
pub enum Expr {
    UnitLit,
    I32Lit(i32),
    BoolLit(bool),
    Lam(Box<Lam>),
    App(Box<Expr>, Vec<Expr>),
    Var(TermVar),
    //FIXME: introduce an If struct to reduce number or allocations
    If(Box<Expr>, Box<Expr>, Box<Expr>, Type ),
    //FIXME: introduce an If struct to reduce number or allocations
    Let(TermVar, Box<Expr>, Box<Expr>),
}

impl Module {
    pub fn new(name: String) -> Self {
        Self{name, types: vec![], funcs: vec![], ext_func: vec![]}
    }

    pub fn name(&self) -> &String {
        &self.name
    }
    
    pub fn types(&self) -> &Vec<Type> {
        &self.types
    }
    
    pub fn funcs(&self) -> &Vec<Func> {
        &self.funcs
    }

    pub fn externs(&self) -> &Vec<FnProto> {
        &self.ext_func
    }

    pub fn add_func(&mut self, lam: Func) {
        self.funcs.push(lam)
    }

    pub fn add_type(&mut self, ty: Type) {
        self.types.push(ty)
    }

    pub fn add_extern(&mut self, proto: FnProto) {
        self.ext_func.push(proto)
    }
}

impl FnProto {
    pub fn new(ident: TermVar, params: Vec<TermVar>) -> Self {
        FnProto{ident, params}
    }
    pub fn name(&self) -> &TermVar {
        &self.ident
    }
    pub fn params(&self) -> &Vec<TermVar> {
        &self.params
    }
    pub fn return_ty(&self) -> &Type {
        self.ident.ty()
    }
}

impl Func {
    pub fn new(name: TermVar, body: Expr) -> Self {
        Func{name, body}
    }
    pub fn proto(&self) -> &TermVar {
        &self.name
    }
    pub fn body(&self) -> &Expr {
        &self.body
    }
}

impl Lam {
    pub fn new(params: Vec<TermVar>, body: Expr) -> Self {
        Lam{params, body}
    }
    pub fn params(&self) -> &Vec<TermVar> {
        &self.params
    }
    pub fn body(&self) -> &Expr {
        &self.body
    }
}

impl TermVar {
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
