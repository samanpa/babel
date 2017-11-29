use ::hir::*;
use ::types::Type;
use ::{Result,Error,VecUtil};


pub struct SimpleTypeChecker {}

impl SimpleTypeChecker {
    pub fn new() -> Self {
        SimpleTypeChecker{}
    }
}

impl ::Pass for SimpleTypeChecker {
    type Input  = Vec<TopLevel>;
    type Output = Vec<TopLevel>;

    fn run(self, toplevel_vec: Self::Input) -> Result<Self::Output> {
        VecUtil::mapt(toplevel_vec, tc_toplevel)
    }
}

fn ty_compare<T: Into<String>>(t1: &Type<u32>, t2: &Type<u32>, msg: T) -> Result<()> {
    if t1 != t2 {
        let msg = format!("Types don't match {}: {:?} != {:?}",
                          msg.into(), t1, t2);
        Err(Error::new(msg))
    }
    else {
        Ok(())
    }    
}


fn tc_toplevel(tl: TopLevel) -> Result<TopLevel> {
    let decls = VecUtil::mapt(tl.decls(), tc_topdecl)?;
    Ok(TopLevel::new(decls))
}
    
fn tc_topdecl(decl: TopDecl) -> Result<TopDecl> {
    use ::hir::TopDecl::*;
    let res = match decl {
        Lam(lam)      => Lam(tc_lam(lam)?),
        Extern(proto) => Extern(tc_proto(&proto)?),
    };
    Ok(res)
}

fn tc_proto(proto: &FnProto) -> Result<FnProto> {
    //FIXME: Check if they are valid types currently not possible to
    //       have invalid types
    Ok(proto.clone())
}

fn tc_lam(lam: Lam) -> Result<Lam> {
    let body_ty = get_type(lam.body())?;
    ty_compare(&body_ty, lam.return_ty()
               , format!("lamba {:?}", lam.ident()))?;
    let expr = tc_expr(lam.body())?;
    let proto = tc_proto(lam.proto())?;
    Ok(Lam::new(proto, expr))
}

fn tc_app(callee: &Expr, args: &Vec<Expr>)-> Result<Type<u32>> {
    use ::types::Type::Function;
    let callee_ty = get_type(callee)?;
    let args_ty   = VecUtil::map(args, get_type)?;
    if let Function{ref params_ty, ref return_ty} = callee_ty {
        if params_ty.len() != args_ty.len() {
            let msg = format!("Invalid number of args to {:?}", callee);
            return Err(Error::new(msg))
        }
        let mut i = 0;
        for (arg, param) in args_ty.iter().zip(params_ty) {
            i += 1;
            let msg = format!("param type {} to {:?}", i, callee);
            ty_compare(arg, param, msg)?
        }
        return Ok((**return_ty).clone());
    }
    return Err(Error::new(format!("Only functions can be applied {:?}"
                                  , callee)))
}
    
fn get_type(expr: &Expr) -> Result<Type<u32>> {
    use ::types::Type::*;
    use ::hir::Expr::*;
    let res = match *expr {
        UnitLit    => Unit,
        I32Lit(_)  => I32,
        BoolLit(_) => Bool,
        Var(ref v) => v.ty().clone(),
        App{ref callee, ref args, ..} => tc_app(callee, args)?,
        If(ref e)  => {
            let cond_ty  = get_type(e.cond())?;
            let texpr_ty = get_type(e.texpr())?;
            let fexpr_ty = get_type(e.fexpr())?;
            ty_compare(&cond_ty, &Bool, "If condition")?;
            ty_compare(&texpr_ty, &fexpr_ty, "True expr and false expr")?;
            texpr_ty
        },
        ref expr   => { println!("NOTHANDLED\n{:?} not handled", expr);
                        unimplemented!() },
    };
    Ok(res)
}

fn tc_expr(expr: &Expr) -> Result<Expr> {
    use ::hir::Expr::*;
    let res = match *expr {
        UnitLit    => UnitLit,
        I32Lit(n)  => I32Lit(n),
        BoolLit(b) => BoolLit(b),
        Var(ref v) => Var(v.clone()),
        App{ref callee, ref args, ref ty_args} => {
            let _ = tc_app(callee, args)?;
            let callee = tc_expr(callee)?;
            let args = VecUtil::map(args, tc_expr)?;
            App{callee: Box::new(callee), args, ty_args: vec![]}
        }
        If(ref e)  => {
            //FIXME: why do I need the self::
            let if_expr = self::If::new(tc_expr(e.cond())?,
                                        tc_expr(e.texpr())?,
                                        tc_expr(e.fexpr())?,
                                        get_type(e.fexpr())?);
            If(Box::new(if_expr))
        },
        ref expr   => { println!("NOTHANDLED\n{:?} not handled", expr);
                        unimplemented!() },
    };
    Ok(res)
}
