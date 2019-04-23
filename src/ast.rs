pub type Type = crate::types::Type<String>;

#[derive(Debug)]
pub struct Module {
    name: String,
    decls: Vec<Decl>,
}

#[derive(Debug)]
pub enum Decl {
    Extern(String, Type),
    Func(Bind),
}

#[derive(Debug)]
pub struct Lam {
    params: Vec<String>,
    body: Expr,
}

#[derive(Debug)]
pub struct If {
    cond: Expr,
    texpr: Expr,
    fexpr: Expr,
}

#[derive(Debug)]
pub struct Bind(pub String, pub Expr);

#[derive(Debug)]
pub enum Expr {
    Lam(Box<Lam>),
    App(Box<Expr>, Vec<Expr>),
    UnitLit,
    I32Lit(i32),
    BoolLit(bool),
    Var(String),
    If(Box<If>),
    Let(Box<Bind>, Box<Expr>),
}

impl Module {
    pub fn new(decls: Vec<Decl>) -> Self {
        Self {
            name: "".to_string(),
            decls,
        }
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn decls(&self) -> &Vec<Decl> {
        &self.decls
    }
}

pub fn con(nm: &str, kind: crate::types::Kind) -> Type {
    use crate::types::TyCon::*;
    let tycon = match nm {
        "i32" => I32,
        "bool" => Bool,
        "()" => Unit,
        "->" => Func,
        _ => NewType(std::rc::Rc::new(nm.to_string())),
    };
    types::Type::Con(tycon, kind)
}

impl Decl {
    pub fn external(name: String, params: Vec<(String, Type)>, retty: Type) -> Self {
        let params_ty: Vec<Type> = params.into_iter().map(|(_, ty)| ty).collect();
        let ty = crate::types::Type::func(params_ty, retty);
        Decl::Extern(name, ty)
    }
}

impl Lam {
    pub fn new(params: Vec<String>, body: Expr) -> Self {
        Lam { params, body }
    }
    pub fn body(&self) -> &Expr {
        &self.body
    }
    pub fn params(&self) -> &Vec<String> {
        &self.params
    }
}

impl If {
    pub fn new(cond: Expr, texpr: Expr, fexpr: Expr) -> Self {
        If { cond, texpr, fexpr }
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
