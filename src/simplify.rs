use crate::monoir;
use crate::types::{Kind, TyCon};
use crate::xir;
use crate::{Error, Result, Vector};

type Type = crate::types::Type<crate::types::TyVar>;

pub struct Simplify {}

impl Default for Simplify {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::Pass for Simplify {
    type Input = Vec<xir::Module>;
    type Output = Vec<monoir::Module>;

    fn run(self, module_vec: Self::Input) -> Result<Self::Output> {
        let res = Vector::map(&module_vec, |modl| self.process(modl))?;
        Ok(res)
    }
}

impl Simplify {
    pub fn new() -> Self {
        Simplify {}
    }

    fn process(&self, module: &xir::Module) -> Result<monoir::Module> {
        let modname = module.name().clone();
        let mut modl = monoir::Module::new(modname);

        for decl in module.decls() {
            match *decl {
                xir::Decl::Extern(ref name) => {
                    modl.ext_funcs.push(process_symbol(name)?);
                }
                xir::Decl::Let(ref bindings) => {
                    for bind in bindings {
                        //println!("{:?} ===========\n  \n", bind);
                        let res = process_bind(bind)?;
                        //println!("{:?}\n====================\n", res);
                        modl.funcs.push(res);
                    }
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

fn process_bind(bind: &xir::Bind) -> Result<monoir::Bind> {
    let sym = process_symbol(bind.symbol())?;
    let expr = process(bind.expr())?;
    let bind = monoir::Bind::new(sym, expr);
    Ok(bind)
}

fn process(expr: &xir::Expr) -> Result<monoir::Expr> {
    use crate::xir::Expr::*;

    let expr = match *expr {
        UnitLit => monoir::Expr::UnitLit,
        I32Lit(n) => monoir::Expr::I32Lit(n),
        BoolLit(b) => monoir::Expr::BoolLit(b),
        Var(ref var) => monoir::Expr::Var(process_symbol(var)?),
        If(ref e) => monoir::Expr::If(Box::new(monoir::If {
            cond: process(e.cond())?,
            texpr: process(e.texpr())?,
            fexpr: process(e.fexpr())?,
            ty: get_type(e.ty())?,
        })),
        Let(ref e) => {
            let bind = process_bind(e.bind())?;
            let expr = process(e.expr())?;
            monoir::Expr::Let(Box::new(bind), Box::new(expr))
        }
        Lam(ref params, ref body, ref _retty) => {
            let params = Vector::map(params, process_symbol)?;
            let body = process(body)?;
            let lam = monoir::Lam::new(params, body);
            monoir::Expr::Lam(Box::new(lam))
        }
        App(ref ty, ref caller, ref args) => {
            let caller = process(caller)?;
            let args = Vector::map(args, process)?;
            monoir::Expr::App(get_type(ty)?, Box::new(caller), args)
        }
        _ => {
            let msg = format!("EXPR not supported {:?}", expr);
            return Err(Error::new(msg));
        }
    };
    Ok(expr)
}

fn get_appty(ty: &Type, args: &[Type]) -> Result<monoir::Type> {
    use crate::types::Type::*;

    let mut args = Vector::map(args, get_type)?;
    match *ty {
        Con(TyCon::Func, _) => {
            if args.is_empty() {
                let msg = format!("Function with no return type found {:?}", ty);
                Err(Error::new(msg))
            } else {
                let slice_end = args.len() - 1; //borrow_chk
                let params_ty = args.drain(..slice_end).collect::<Vec<_>>();
                let return_ty = Box::new(args.pop().unwrap());
                Ok(monoir::Type::Function {
                    params_ty,
                    return_ty,
                })
            }
        }
        _ => {
            let msg = format!("not supported {:?}", ty);
            Err(Error::new(msg))
        }
    }
}

fn get_type(ty: &Type) -> Result<monoir::Type> {
    use self::Kind::*;
    use self::TyCon::*;
    use crate::types::Type::*;
    let ty = match ty {
        App(ty, args) => get_appty(ty, args)?,
        Con(tycon, k) => match (tycon, k) {
            (&I32, &Star) => monoir::Type::I32,
            (&Bool, &Star) => monoir::Type::Bool,
            (&Unit, &Star) => monoir::Type::Unit,
            _ => {
                let msg = format!("not supported {:?}", ty);
                return Err(Error::new(msg));
            }
        },
        _ => {
            let msg = format!("not supported {:?}", ty);
            return Err(Error::new(msg));
        }
    };
    Ok(ty)
}
