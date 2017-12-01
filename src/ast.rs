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

#[derive(Debug)]
pub struct FnProto {
    name: String,
    params: Vec<String>,
    ty: ::types::Function<String>,
}

pub type Type = ::types::Type<String>;

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
    Var(String),
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
        let mut params_ty = Vec::with_capacity(params.len());
        let mut params_nm = Vec::with_capacity(params.len());
        for p in params {
            params_nm.push(p.name);
            params_ty.push(p.ty);
        }
        let ty_vars = ty_vars.into_iter()
            .map( |ty_var| ::types::Type::TyVar(ty_var))
            .collect();
        let ty = ::types::Function::new(ty_vars, params_ty, return_ty);
        FnProto{name, params: params_nm, ty}
    }
    pub fn name(&self) -> &String {
        &self.name
    }
    pub fn params(&self) -> &Vec<String> {
        &self.params
    }
    pub fn ty(&self) -> &::types::Function<String> {
        &self.ty
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
        &self.proto.ty().return_ty()
    }
    pub fn ty(&self) -> &::types::Function<String> {
        self.proto.ty()
    }
    pub fn params(&self) -> &Vec<String> {
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
