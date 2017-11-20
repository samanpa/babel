use std::rc::Rc;

#[derive(Debug)]
pub struct TopLevel {
    decls: Vec<TopDecl>,
}

#[derive(Debug)]
pub struct FnProto {
    ident: Ident,
    params: Vec<Ident>,
    return_ty: Type,
    ty_vars: Vec<u32>,
}

#[derive(Debug)]
pub enum TopDecl {
    Extern(FnProto),
    Lam(Lam),
}

#[derive(Debug,Clone)]
pub struct Ident {
    name: Rc<String>,
    id: u32,
    ty: Type,
}

#[derive(Debug,Clone)]
pub enum Type {
    Bool,
    I32,
    Unit,
    Function{ params_ty: Vec<Type>, return_ty: Box<Type> },
    TyVar(u32),
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
    Lam(Box<Lam>),
    App{callee: Box<Expr>, args: Vec<Expr> },
    UnitLit,
    I32Lit(i32),
    BoolLit(bool),
    Var(Ident),
    If(Box<If>)
}

impl TopLevel {
    pub fn new(decls: Vec<TopDecl>) -> Self {
        Self{decls}
    }

    pub fn decls(&self) -> &Vec<TopDecl> {
        &self.decls
    }
    pub fn decls_mut(&mut self) -> &mut Vec<TopDecl> {
        &mut self.decls
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
    pub fn new(ident: Ident, params: Vec<Ident>, ty_vars: Vec<u32>
               , return_ty: Type) -> Self {
        FnProto{ ident, params, ty_vars, return_ty }
    }
    pub fn ident(&self) -> &Ident {
        &self.ident
    }
    pub fn return_ty(&self) -> &Type {
        &self.return_ty
    }
    pub fn params(&self) -> &Vec<Ident> {
        &self.params
    }
    pub fn ty_vars(&self) -> &Vec<u32> {
        &self.ty_vars
    }
}

impl PartialEq for Type {
    fn eq(&self, rhs: &Type) -> bool {
        use self::Type::*;
        match (self, rhs) {
            (&Bool, &Bool) => true,
            (&I32,  &I32)  => true,
            (&Unit, &Unit) => true,
            (&TyVar(_), _) => true,
            (_, &TyVar(_)) => true,
            (&Function{params_ty: ref lparam, return_ty: ref lreturn},
             &Function{params_ty: ref rparam, return_ty: ref rreturn}) => {
                lreturn == rreturn &&
                    lparam.len() == rparam.len() &&
                    lparam.iter().zip(rparam)
                       .fold(true, |prev, (lty, rty)| prev && (lty == rty))
            },
            _ => false,
        }
    }
}

impl Lam {
    pub fn new(proto: FnProto, body: Expr) -> Self {
        Lam{proto, body}
    }
    pub fn proto(&self) -> &FnProto {
        &self.proto
    }
    pub fn proto1(self) -> FnProto {
        self.proto
    }
    pub fn body(&self) -> &Expr {
        &self.body
    }
    pub fn body_mut(&mut self) -> &mut Expr {
        &mut self.body
    }
    pub fn ident(&self) -> &Ident {
        &self.proto.ident
    }
    pub fn return_ty(&self) -> &Type {
        &self.proto.return_ty
    }
    pub fn params(&self) -> &Vec<Ident> {
        &self.proto.params
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
