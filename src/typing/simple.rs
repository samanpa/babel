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
    if ::types::unifiable(t1, t2) == false {
        let msg = format!("Types are not unifiable in {}:\n\t{:?}\n\t{:?}",
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
    ty_compare(&body_ty, lam.proto().ty().return_ty()
               , format!("lamba {:?}", lam.ident()))?;
    let (expr, _)  = tc_expr(lam.body())?;
    let proto = tc_proto(lam.proto())?;
    Ok(Lam::new(proto, expr))
}

fn tc_app(callee: &Expr, args: &Vec<Expr>)-> Result<::types::Function<u32>> {
    let callee_ty = get_type(callee)?;
    let args_ty   = VecUtil::map(args, get_type)?;
    if let ::types::Type::Func(ref func_ty) = callee_ty {
        let params_ty = func_ty.params_ty();
        let return_ty = func_ty.return_ty();
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
        return Ok(::types::Function::new(func_ty.ty_vars().clone()
                                         , params_ty.clone()
                                         , return_ty.clone()));
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
        Var(ref v, ref _ty) => v.ty().clone(), //FIXME: do substition
        App{ref callee, ref args, ..} => tc_app(callee, args)?.return_ty().clone(),
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

fn tc_expr(expr: &Expr) -> Result<(Expr,Type<u32>)> {
    use super::explicate::infer_ty_args;
    use ::hir::Expr::*;
    use ::types::Type::*;

    let res = match *expr {
        UnitLit    => (UnitLit, Unit),
        I32Lit(n)  => (I32Lit(n), I32),
        BoolLit(b) => (BoolLit(b), Bool),
        Var(ref v, ref ty) => (Var(v.clone(), ty.clone()), v.ty().clone()),
        App{ref callee, ref args} => {
            let callee_ty = tc_app(callee, args)?;
            let (mut callee, _) = tc_expr(callee)?;
            let args    = VecUtil::map(args, |arg| Ok(tc_expr(arg)?.0))?;
            let app = App{callee: Box::new(callee), args};
            (app, callee_ty.return_ty().clone())
        }
        If(ref e)  => {
            let ty = get_type(expr)?;
            let (cond, _)   = tc_expr(e.cond())?;
            let (texpr, _)  = tc_expr(e.texpr())?;
            let (fexpr, ty) = tc_expr(e.fexpr())?;
            
            //FIXME: why do I need the self::
            let if_expr = self::If::new(cond, texpr, fexpr, ty.clone());
            (If(Box::new(if_expr)), ty)
        },
        ref expr   => { println!("NOTHANDLED\n{:?} not handled", expr);
                        unimplemented!() },
    };
    Ok(res)
}
