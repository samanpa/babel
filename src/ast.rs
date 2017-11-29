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
    params: Vec<Param>,
    return_ty: Type,
    ty_vars: Vec<String>,
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
    App{callee: Box<Expr>, ty_args: Vec<Type>, args: Vec<Expr> },
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
        FnProto{name, params, return_ty, ty_vars}
    }
    pub fn name(&self) -> &String {
        &self.name
    }
    pub fn return_ty(&self) -> &Type {
        &self.return_ty
    }
    pub fn params(&self) -> &Vec<Param> {
        &self.params
    }
    pub fn ty_vars(&self) -> &Vec<String> {
        &self.ty_vars
    }
    pub fn ty(&self) -> Type {
        let params_ty = self.params.iter()
            .map(|ref param| param.ty.clone())
            .collect();
        let return_ty = Box::new(self.return_ty.clone());
        ::types::Type::Function{ params_ty, return_ty }
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
        &self.proto.return_ty
    }
    pub fn ty(&self) -> Type {
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
