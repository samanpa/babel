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

    fn run(self, module_vec: Self::Input) -> Result<Self::Output> {
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
        let var  = process_termvar(var)?;
        let mut args = Vec::new();
        let body = process(body, &mut args)?;
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
                    println!("{:?} ===========\n  {:?}\n", id, expr);
                    let res = self.func(id, expr)?;
                    println!("{:?}\n====================\n", res);
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


fn process(expr: &xir::Expr, mut args: &mut Vec<monoir::Expr>)
           -> Result<monoir::Expr>
{
    use xir::Expr::*;

    let expr = match *expr {
        UnitLit      => monoir::Expr::UnitLit,
        I32Lit(n)    => monoir::Expr::I32Lit(n),
        BoolLit(b)   => monoir::Expr::BoolLit(b),
        Var(ref var) => monoir::Expr::Var(process_termvar(var)?),
        If(ref e) => {
            monoir::Expr::If(Box::new(process(e.cond(),  args)?),
                             Box::new(process(e.texpr(), args)?),
                             Box::new(process(e.fexpr(), args)?),
                             get_type(e.ty())?)
        }
        Let(ref e) => {
            monoir::Expr::Let(process_termvar(e.id())?,
                              Box::new(process(e.bind(), args)?),
                              Box::new(process(e.expr(), args)?))
        }
        Lam(ref params, ref body) => {
            let params = Vector::map(params, process_termvar)?;
            let body   = process(body, args)?;
            let lam    = monoir::Lam::new(params, body);
            monoir::Expr::Lam(Box::new(lam))
        }
        App(1, ref caller, ref arg) => {
            let mut args = Vec::with_capacity(2);
            let caller   = process(caller, &mut args)?;
            let arg      = process(arg, &mut args)?;
            args.push(arg);
            monoir::Expr::App(Box::new(caller), args)
        }
        App(_, ref caller, ref arg) => {
            let caller = process(caller, args)?;
            let arg    = process(arg, args)?;
            args.push(arg);
            caller
        }
        _ => {
            let msg = format!("EXPR not supported {:?}", expr);
            return Err(Error::new(msg))
        }
    };
    Ok(expr)
}

struct FuncTy<'t> {
    nargs: u32,
    first_arg: &'t Type,
    rest: &'t Type,
}

fn get_functy<'t>(ty: &'t Type) -> Option<FuncTy<'t>> {
    use self::Type::*;
    if let App(ref caller, ref rest) = *ty {
        if let App(ref caller2, ref first_arg) = **caller {
            match **caller2 {
                Con(ref name, nargs) if name.as_ref() == "->" => { 
                    let res = FuncTy{nargs, first_arg, rest};
                    return Some(res)
                }
                _ => {}
            }
        }
    }
    None
}

fn process_fnty(arg_cnt:u32, ret: &Type, args: &mut Vec<monoir::Type>)
                -> Result<monoir::Type>
{
    println!("Process {:?} {:?}", arg_cnt, ret);
    match arg_cnt {
        1 => get_type(ret),
        n => {
            match get_functy(ret) {
                Some(fty) =>  {
                    let p = get_type(fty.first_arg)?;
                    args.push(p);
                    process_fnty(fty.nargs-1, fty.rest, args)
                }
                None => {
                    let msg = format!("not a functy {:?}", ret);
                    return Err(Error::new(msg));                        
                }
            }
        }
    }
}

fn get_appty(caller: &Type, arg: &Type) -> Result<monoir::Type> {
    use self::Type::*;
    if let App(ref caller2, ref arg2) = *caller {
        let first_param  = get_type(arg2)?;
        match **caller2 {
            Con(ref name, n) if name.as_ref() == "->" => { 
                let mut params_ty = Vec::with_capacity(n as usize);
                params_ty.push(first_param);
                let return_ty = process_fnty(n-1, arg, &mut params_ty)?;
                let return_ty = Box::new(return_ty);
                return Ok(monoir::Type::Function{ params_ty, return_ty });
            }
            _ => {}
        }
    }

    let msg = format!("not supported {:?} {:?}", caller, arg);
    Err(Error::new(msg))
}

fn get_type(ty: &Type) -> Result<monoir::Type> {
    use self::Type::*;
    let ty = match *ty {
        App(ref caller, ref arg) => {
            get_appty(caller, arg)?
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
