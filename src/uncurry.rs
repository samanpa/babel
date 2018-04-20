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

    fn func(&self, f: &xir::Bind) -> Result<monoir::Bind>
    {
        match *f {
            xir::Bind::NonRec{ref symbol, ref expr} => {
                let sym      = process_symbol(symbol)?;
                let mut args = Vec::new();
                let expr     = process(expr, &mut args)?;
                Ok(monoir::Bind::new(sym, expr))
            }
        }
    }
    
    fn uncurry_module(&self, module: &xir::Module) -> Result<monoir::Module>
    {
        let modname  = module.name().clone();
        let mut modl = monoir::Module::new(modname);

        for decl in module.decls() {
            match *decl {
                xir::Decl::Extern(ref name) => {
                    modl.add_extern(process_symbol(name)?);
                },
                xir::Decl::Let(ref bind) => {
                    //println!("{:?} ===========\n  \n", bind);
                    let res = self.func(bind)?;
                    //println!("{:?}\n====================\n", res);
                    modl.add_func(res);
                }
            }
        }

        Ok(modl)
    }
}

fn process_symbol(sym: &xir::Symbol) -> Result<monoir::Symbol> {
    let ty = get_type(sym.ty())?;
    let tv = monoir::Symbol::new(sym.name().clone(), ty, sym.id());
    Ok(tv)
}


fn process_bind(bind: &xir::Bind, args: &mut Vec<monoir::Expr>)
                -> Result<monoir::Bind>
{
    let bind = match *bind {
        xir::Bind::NonRec{ref symbol, ref expr} => {
            let sym  = process_symbol(symbol)?;
            let expr = process(expr, args)?;
            monoir::Bind::new(sym, expr)
        }
    };
    Ok(bind)
}

fn process(expr: &xir::Expr, args: &mut Vec<monoir::Expr>)
           -> Result<monoir::Expr>
{
    use xir::Expr::*;

    let expr = match *expr {
        UnitLit      => monoir::Expr::UnitLit,
        I32Lit(n)    => monoir::Expr::I32Lit(n),
        BoolLit(b)   => monoir::Expr::BoolLit(b),
        Var(ref var) => monoir::Expr::Var(process_symbol(var)?),
        If(ref e) => {
            monoir::Expr::If(Box::new(process(e.cond(),  args)?),
                             Box::new(process(e.texpr(), args)?),
                             Box::new(process(e.fexpr(), args)?),
                             get_type(e.ty())?)
        }
        Let(ref e) => {
            let bind = process_bind(e.bind(), args)?;
            let expr = process(e.expr(), args)?;
            monoir::Expr::Let(Box::new(bind), Box::new(expr))
        }
        Lam(ref params, ref body, ref _retty) => {
            let params = Vector::map(params, process_symbol)?;
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

fn get_appty(ty: &Type) -> Result<monoir::Type> {
    if let Some(FuncTy{nargs, first_arg, rest}) = get_functy(ty) {
        let mut params_ty = Vec::with_capacity(nargs as usize);
        params_ty.push(get_type(first_arg)?);
        let return_ty = process_fnty(nargs-1, rest, &mut params_ty)?;
        let return_ty = Box::new(return_ty);
        return Ok(monoir::Type::Function{ params_ty, return_ty });
    }

    let msg = format!("not supported {:?}", ty);
    Err(Error::new(msg))
}

fn get_type(ty: &Type) -> Result<monoir::Type> {
    use self::Type::*;
    let ty = match *ty {
        App(_, _) => get_appty(ty)?,
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
