#[derive(Debug)]
pub struct TopLevel {
    decls: Vec<TopDecl>,
}

#[derive(Debug)]
pub enum TopDecl {
    Extern(FnProto),
    Use(String),
    Lam(Lam),
}

pub type ForAll = ::types::ForAll<String>;
pub type Type = ::types::Type<String>;

#[derive(Debug)]
pub struct FnProto {
    name: String,
    params: Vec<Param>,
    ty: ForAll,
}

#[derive(Debug)]
pub struct Param {
    name: String,
    ty: Type,
}

#[derive(Debug)]
pub struct Lam {
    proto: FnProto,
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
    App{callee: Box<Expr>, args: Vec<Expr>},
    UnitLit,
    I32Lit(i32),
    BoolLit(bool),
    Var(String, Vec<Type>),
    If(Box<If>)
}

impl TopLevel {
    pub fn new(decls: Vec<TopDecl>) -> Self {
        Self{decls}
    }

    pub fn decls(&self) -> &Vec<TopDecl> {
        &self.decls
    }
}

impl FnProto {
    pub fn new(name: String, params: Vec<Param>, return_ty: Type
               , ty_vars: Vec<String> ) -> Self {
        use ::types::*;
        let params_ty = params.iter()
            .map( |p| p.ty.clone() )
            .collect();
        let fn_ty = Type::Func(Box::new(Function::new(params_ty, return_ty)));
        let ty    = ::types::ForAll::new(ty_vars, fn_ty);
        FnProto{name, params, ty}
    }
    pub fn name(&self) -> &String {
        &self.name
    }
    pub fn params(&self) -> &Vec<Param> {
        &self.params
    }
    pub fn ty(&self) -> &ForAll {
        &self.ty
    }
    pub fn return_ty(&self) -> &Type {
        if let ::types::Type::Func(ref f) = *self.ty.ty() {
            return f.return_ty()
        }
        panic!("A function proto should always have a func type")
    }
}

impl Lam {
    pub fn new(name: String, params: Vec<Param>, return_ty: Type
               , ty_vars: Vec<String>, body: Expr) -> Self {
        let proto = FnProto::new(name, params, return_ty, ty_vars);
        Lam{proto, body}
    }
    pub fn proto(&self) -> &FnProto {
        &self.proto
    }
    pub fn body(&self) -> &Expr {
        &self.body
    }
    pub fn name(&self) -> &String {
        &self.proto.name
    }
    pub fn return_ty(&self) -> &Type {
        &self.proto.return_ty()
    }
    pub fn ty(&self) -> &ForAll {
        self.proto.ty()
    }
    pub fn params(&self) -> &Vec<Param> {
        &self.proto.params
    }
}

impl Param {
    pub fn new(name: String, ty: Type) -> Self {
        Param{name, ty}
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn ty(&self) -> &Type {
        &self.ty
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
