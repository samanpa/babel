use std::rc::Rc;
pub type Type = ::types::Type<u32>;
pub type ForAll = ::types::ForAll<u32>;

#[derive(Debug)]
pub struct TopLevel {
    decls: Vec<TopDecl>,
}

#[derive(Debug,Clone)]
pub struct FnProto {
    ident: Ident,
    params: Vec<Ident>,
    ty: ForAll,
}

#[derive(Debug)]
pub enum TopDecl {
    Extern(FnProto),
    Lam(Rc<Lam>),
}

#[derive(Debug,Clone)]
pub struct Ident {
    name: Rc<String>,
    id: u32,
    ty: Type,
}


#[derive(Debug)]
pub struct Lam {
    proto: FnProto,
    body:  Expr,
}

#[derive(Debug)]
pub struct If {
    cond:  Expr,
    texpr: Expr,
    fexpr: Expr,
    res_ty: Type,
}

#[derive(Debug)]
pub enum Expr {
    Lam(Rc<Lam>),
    App(Box<Expr>, Vec<Expr>),
    UnitLit,
    I32Lit(i32),
    BoolLit(bool),
    Var(Ident, Vec<Type>),
    If(Box<If>)
}

impl TopLevel {
    pub fn new(decls: Vec<TopDecl>) -> Self {
        Self{decls}
    }

    pub fn decls(self) -> Vec<TopDecl> {
        self.decls
    }

    pub fn add_decl(&mut self, decl: TopDecl) {
        self.decls.push(decl)
    }
}

impl Ident {
    pub fn new(name: Rc<String>, ty: Type, id: u32) -> Self {
        Ident{name, ty, id}
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

impl FnProto {
    pub fn new(ident: Ident, params: Vec<Ident>, ty: ForAll) -> Self {
        FnProto{ ident, params, ty }
    }
    pub fn ident(&self) -> &Ident {
        &self.ident
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
    pub fn params(&self) -> &Vec<Ident> {
        &self.params
    }
}

impl Lam {
    pub fn new(proto: FnProto, body: Expr) -> Self {
        Lam{proto, body}
    }
    pub fn proto(&self) -> &FnProto {
        &self.proto
    }
    pub fn take_proto(self) -> FnProto {
        self.proto
    }
    pub fn body(&self) -> &Expr {
        &self.body
    }
    pub fn ident(&self) -> &Ident {
        &self.proto.ident
    }
}

impl If {
    pub fn new(cond: Expr, texpr: Expr, fexpr: Expr, res_ty: Type) -> Self {
        If{cond, texpr, fexpr, res_ty}
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
    pub fn res_ty(&self) -> &Type {
        &self.res_ty
    }
}
