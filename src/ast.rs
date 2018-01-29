#[derive(Debug)]
pub enum Type {
    App(Box<Type>, Box<Type>),
    Con(String, u32),
    Var(String)
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
    App(u32, Box<Expr>, Box<Expr>),
    UnitLit,
    I32Lit(i32),
    BoolLit(bool),
    Var(String),
    If(Box<If>),
    Let(String, Box<Expr>, Box<Expr>)
}

impl Type {
    pub fn arity(&self) -> u32 {
        match *self {
            Type::App(ref l, _) => l.arity() - 1,
            Type::Con(_, arity) => arity,
            Type::Var(_)        => 0, //FIXME: not true when we have HKT
        }
    }
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

pub fn mk_app(expr: Expr, args: Vec<Expr>) -> Expr {
    let app = |(expr,n), arg| {
        let expr = Expr::App(n, Box::new(expr), Box::new(arg));
        (expr, n-1)
    };
    let (expr, _) = match args.len() as u32 {
        0 => app((expr,1), Expr::UnitLit),
        n => args.into_iter().fold((expr,n), app)
    };
    expr
}        

pub fn make_func(param: Vec<Type>, ret: Type) -> Type {
    use self::Type::*;
    let mk_fn = |(ret, arity), param| {
        let con  = Box::new(Con("->".to_string(), arity));
        let app1 = App(con, Box::new(param));
        let func = App(Box::new(app1), Box::new(ret));
        (func, arity + 1)
    };
    let itr = param.into_iter().rev();
    let (ty, _arity) = match itr.len() as u32 {
        0 => mk_fn((ret, 2), Con("unit".to_string(), 0)),
        _ => itr.fold((ret, 2), mk_fn),
    };
    ty
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
