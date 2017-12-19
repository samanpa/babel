#[derive(Debug)]
pub enum Type {
    TyApp(Box<Type>, Vec<Type>),
    TyCon(String),
    TyVar(String)
}

#[derive(Debug)]
pub struct Module {
    name: String,
    decls: Vec<Decl>,
}

#[derive(Debug)]
pub enum Decl {
    Extern(String, Type),
    Func(String, Lam),
}

#[derive(Debug)]
pub struct Lam {
    params: Vec<String>,
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
    App(Box<Expr>, Box<Expr>),
    UnitLit,
    I32Lit(i32),
    BoolLit(bool),
    Var(String),
    If(Box<If>),
    Let(String, Box<Expr>, Box<Expr>)
}

impl Module {
    pub fn new(decls: Vec<Decl>) -> Self {
        Self{name: "".to_string(), decls}
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

pub fn make_func(param: Vec<Type>, ret: Type) -> Type {
    use self::Type::*;
    let mk_fn = |ret, param| TyApp(Box::new(TyCon("->".to_string()))
                                   , vec![param, ret]);

    let itr = param.into_iter().rev();
    let ty = match itr.len() {
        0 => mk_fn(ret, TyCon("unit".to_string())),
        _ => itr.fold(ret, mk_fn),
    };
    ty
}

pub fn mk_app(expr: Expr, args: Vec<Expr>) -> Expr {
    let app = |expr, arg| Expr::App(Box::new(expr), Box::new(arg));

    match args.len() {
        0 => app(expr, Expr::UnitLit),
        _ => args.into_iter().fold(expr, app)
    }
}        

impl Decl {
    pub fn external(name: String, params: Vec<(String, Type)>,
                    retty: Type) -> Self {
        let params_ty : Vec<Type> = params.into_iter()
            .map(|(_,ty)| ty)
            .collect();
        let ty = make_func(params_ty, retty);
        Decl::Extern(name, ty)
    }
}

impl Lam {
    pub fn new(params: Vec<String>, body: Expr) -> Self {
        Lam{params, body}
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
