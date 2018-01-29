use xir;
use typing::types::Type;
use monoir;
use {Result,Vector,Error};

pub struct Uncurry {}

impl Default for Uncurry {
    fn default() -> Self {
        Self::new()
    }
}

impl ::Pass for Uncurry {
    type Input  = Vec<xir::Module>;
    type Output = Vec<monoir::Module>;

    fn run(mut self, module_vec: Self::Input) -> Result<Self::Output> {
        let res = Vector::map(&module_vec, |modl| self.uncurry_module(modl))?;
        Ok(res)
    }
}

impl Uncurry {
    pub fn new() -> Self {
        Uncurry{}
    }

    fn func(&self, var: &xir::TermVar, body: &xir::Expr)
            -> Result<monoir::Func>
    {
        let var = process_termvar(var)?;
        let body = process(body)?;
        Ok(monoir::Func::new(var, body))
    }
    
    fn uncurry_module(&self, module: &xir::Module)
                      -> Result<monoir::Module>
    {
        let modname  = module.name().clone();
        let mut modl = monoir::Module::new(modname);

        for decl in module.decls() {
            match *decl {
                xir::Decl::Extern(_, _) => (),
                xir::Decl::Let(ref id, ref expr) => {
                    println!("{:?} =\n  {:?}\n", id, expr);
                    let res = self.func(id, expr)?;
                    println!("{:?}\n==========\n", res);
                    modl.add_func(res);
                }
            }
        }

        //let decls   = Vec::new();
        Ok(modl)
    }
}

fn process_termvar(termvar: &xir::TermVar) -> Result<monoir::TermVar> {
    let ty = get_type(termvar.ty())?;
    let tv = monoir::TermVar::new(termvar.name().clone(), ty, termvar.id());
    Ok(tv)
}

fn process(expr: &xir::Expr) -> Result<monoir::Expr> {
    use xir::Expr::*;

    let expr = match *expr {
        UnitLit      => monoir::Expr::UnitLit,
        I32Lit(n)    => monoir::Expr::I32Lit(n),
        BoolLit(b)   => monoir::Expr::BoolLit(b),
        Var(ref var) => monoir::Expr::Var(process_termvar(var)?),
        If(ref e) => {
            monoir::Expr::If(Box::new(process(e.cond())?),
                             Box::new(process(e.texpr())?),
                             Box::new(process(e.fexpr())?),
                             get_type(e.ty())?)
        }
        Let(ref e) => {
            monoir::Expr::Let(process_termvar(e.id())?,
                              Box::new(process(e.bind())?),
                              Box::new(process(e.expr())?))
        }
        Lam(ref params, ref body) => {
            let params = Vector::map(params, process_termvar)?;
            let body   = process(body)?;
            let lam    = monoir::Lam::new(params, body);
            monoir::Expr::Lam(Box::new(lam))
        }
        App(n, ref callee, ref arg) => {
            //let callee = self.run(callee, sub, vec![])?;
            //let arg    = self.run(arg, sub, vec![])?;
            //xir::Expr::App(n, Box::new(callee), Box::new(arg))
            monoir::Expr::UnitLit
        }
        _ => {
            let msg = format!("EXPR not supported {:?}", expr);
            return Err(Error::new(msg))
        }
    };
    Ok(expr)
}

fn process_fnty(arg_cnt:u32, param: &Type, ret: &Type) -> Result<monoir::Type>
{
    let param = get_type(param)?;
    let ret   = get_type(ret)?;

    Ok(monoir::Type::Function{ params_ty:   vec![param]
                               , return_ty: Box::new(ret) })
}

fn get_appty(callee: &Type, arg: &Type) -> Result<monoir::Type> {
    use self::Type::*;
    if let App(ref callee2, ref arg2) = *callee { 
        match **callee2 {
            Con(ref name, n) if name.as_ref() == "->" => { 
                return process_fnty(n, arg, arg2)
            }
            _ => {}
        }
    }

    let msg = format!("not supported {:?} {:?}", callee, arg);
    Err(Error::new(msg))
}

fn get_type(ty: &Type) -> Result<monoir::Type> {
    use self::Type::*;
    let ty = match *ty {
        App(ref callee, ref arg) => {
            get_appty(callee, arg)?
        }
        Con(ref name, n) => {
            match (name.as_str(), n) {
                ("i32",  0) => monoir::Type::I32,
                ("bool", 0) => monoir::Type::Bool,
                ("unit", 0) => monoir::Type::Unit,
                _           => {
                    let msg = format!("not supported {:?}", ty);
                    return Err(Error::new(msg))
                }
            }
        }
        _ => {
            let msg = format!("not supported {:?}", ty);
            return Err(Error::new(msg))
        }
    };
    Ok(ty)
}

/*
fn mk_func(param: &Vec<Type>, ret: Type) -> Type {
    use self::Type::*;
    let mk_fn = |ret, param: &Type| App(Box::new(mk_tycon("->"))
                                        , vec![param.clone(), ret]);
    
    let itr = param.into_iter().rev();
    match itr.len() {
        0 => mk_fn(ret, &mk_tycon("unit")),
            _ => itr.fold(ret, mk_fn),
    }
}

    
fn monoir_fnty(ty: &xir::Type, mut nparams: n) -> Result<monoir::Type> {
    let params = Vec::with_capacity(n);
    let set_i  = |i, ty| unsafe{params.unchecked_set(i, monoir_type(ty)) }
    let mut itr = ty;
    while n > 0 {
        match *itr => {
            Type::App(ref 
        
    }
fn monoir_type(ty: &xir::Type) -> Result<monoir::Type> {
    match *ty {
        Type::Con( ref tyname) => {
            match *tyname => {
                "i32"  => monoir::Type::i32,
                "bool" => monoir::Type::Bool,
                "unit" => monoir::Type::Unit,
                _      => panic!("Not supported"),
            },
        }
        Type::App(_, _) => {
            panic!("App Not supported"),
        }
        Type::Var(_) => {
            panic!("Var not suppored"),
        }
    }                    
}
    
*/
