use std::rc::Rc;

//FIXME: is Module a good name?
#[derive(Debug)]
pub struct Module {
    name:  String,
    types: Vec<Type>,
    lambdas: Vec<Lambda>,
    ext_func: Vec<FnProto>,
}

#[derive(Debug,Clone)]
pub enum Type {
    Unit,
    Bool,
    I32,
    Function{ params_ty: Vec<Type>, return_ty: Box<Type> },
}

#[derive(Debug)]
pub struct Ident {
    name: Rc<String>,
    ty:   Type,
    id:   u32,
}

#[derive(Debug)]
pub struct FnProto {
    ident: Ident,
    params: Vec<Ident>
}

#[derive(Debug)]
pub enum Literal {
    Unit,
    I32(i32),
    Bool(bool),
}

#[derive(Debug)]
pub struct Lambda {
    proto: FnProto,
    body: Expr,
}

#[derive(Debug)]
pub enum Expr {
    //Convert these into Type constants ?
    UnitLit,
    I32Lit(i32),
    BoolLit(bool),

    Lambda(Box<Lambda>),
    App{callee: Box<Expr>, args: Vec<Expr> },
    Var(Ident),
    //FIXME: introduce an If struct to reduce number or allocations
    If{cond: Box<Expr>, texpr: Box<Expr>, fexpr: Box<Expr>, ty: Type }
}

impl Module {
    pub fn new(name: String) -> Self {
        Self{name, types: vec![], lambdas: vec![], ext_func: vec![]}
    }

    pub fn name(&self) -> &String {
        &self.name
    }
    
    pub fn types(&self) -> &Vec<Type> {
        &self.types
    }
    
    pub fn lambdas(&self) -> &Vec<Lambda> {
        &self.lambdas
    }

    pub fn externs(&self) -> &Vec<FnProto> {
        &self.ext_func
    }

    pub fn add_lambda(&mut self, lam: Lambda) {
        self.lambdas.push(lam)
    }

    pub fn add_type(&mut self, ty: Type) {
        self.types.push(ty)
    }

    pub fn add_extern(&mut self, proto: FnProto) {
        self.ext_func.push(proto)
    }
}

impl FnProto {
    pub fn new(ident: Ident, params: Vec<Ident>) -> Self {
        FnProto{ident, params}
    }
    pub fn name(&self) -> &Ident {
        &self.ident
    }
    pub fn params(&self) -> &Vec<Ident> {
        &self.params
    }
    pub fn return_ty(&self) -> &Type {
        self.ident.ty()
    }
}

impl Lambda {
    pub fn new(proto: FnProto, body: Expr) -> Self {
        Lambda{proto, body}
    }
    pub fn proto(&self) -> &FnProto {
        &self.proto
    }
    pub fn body(&self) -> &Expr {
        &self.body
    }
}

impl Ident {
    pub fn new(name: Rc<String>, ty: Type, id: u32) -> Self {
        Self{name, ty, id}
    }
    pub fn name(&self) -> &String {
        &self.name
    }
    pub fn id(&self) -> u32 {
        self.id
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
