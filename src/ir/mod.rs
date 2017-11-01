use std::rc::Rc;

//FIXME: is Module a good name?
#[derive(Debug)]
pub struct Module {
    name:  String,
    types: Vec<Type>,
    lambdas: Vec<Lambda>,
    ext_func: Vec<FnProto>,
}

#[repr(i8)]
#[derive(PartialEq,Eq,Clone,Copy,Debug)]
pub enum BaseType {
    Unit,
    Bool,
    I32
}

#[derive(Debug)]
pub enum Type {
    BaseType(BaseType),
    FunctionType{ params_ty: Vec<Type>, return_ty: Box<Type> }
}

#[derive(Debug)]
pub struct Var {
    name: Rc<String>,
    ty:   Type,
    id:   u32,
}

impl Var {
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

#[derive(Debug)]
//FIXME: A Var already has an associated type so the return_ty is unnecessary.
//  also params has duplicate type name for each parameter
pub struct FnProto {
    name: Var,
    params: Vec<Var>,
    return_ty: Type
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
    Var(Var),
    If{cond: Box<Expr>, texpr: Box<Expr>, fexpr: Box<Expr> }
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
    pub fn new(name: Var, params: Vec<Var>, return_ty: Type) -> Self {
        FnProto{name, params, return_ty}
    }
    pub fn name(&self) -> &Var {
        &self.name
    }
    pub fn return_ty(&self) -> &Type {
        &self.return_ty
    }
    pub fn params(&self) -> &Vec<Var> {
        &self.params
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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
