use ::hir::*;
use ::types::Type;
use ::{Result,Error,VecUtil};


pub struct SimpleTypeChecker {}

impl ::Pass for SimpleTypeChecker {
    type Input  = Vec<TopLevel>;
    type Output = Vec<TopLevel>;

    fn run(self, toplevel_vec: Self::Input) -> Result<Self::Output> {
        VecUtil::mapt(toplevel_vec, |tl| self.tc_toplevel(tl))
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

impl SimpleTypeChecker {
    pub fn new() -> Self {
        SimpleTypeChecker{}
    }

    fn tc_toplevel(&self, tl: TopLevel) -> Result<TopLevel> {
        let decls = VecUtil::mapt(tl.decls(), |decl| self.tc_topdecl(decl))?;
        Ok(TopLevel::new(decls))
    }
    
    fn tc_topdecl(&self, decl: TopDecl) -> Result<TopDecl> {
        use ::hir::TopDecl::*;
        let res = match decl {
            Lam(lam)  => Lam(self.tc_lam(lam)?),
            Extern(proto) => Extern(self.tc_proto(&proto)?),
        };
        Ok(res)
    }

    fn tc_proto(&self, proto: &FnProto) -> Result<FnProto> {
        //FIXME: Check if they are valid types currently not possible to
        //       have invalid types
        Ok(proto.clone())
    }

    fn tc_lam(&self, lam: Lam) -> Result<Lam> {
        let body_ty = self.get_type(lam.body())?;
        ty_compare(&body_ty, lam.return_ty()
                   , format!("lamba {:?}", lam.ident()))?;
        let expr = self.tc_expr(lam.body())?;
        let proto = self.tc_proto(lam.proto())?;
        Ok(Lam::new(proto, expr))
    }

    fn get_type(&self, expr: &Expr) -> Result<Type<u32>> {
        use ::types::Type::*;
        use ::hir::Expr::*;
        let res = match *expr {
            UnitLit    => Unit,
            I32Lit(_)  => I32,
            BoolLit(_) => Bool,
            Var(ref v) => v.ty().clone(),
            App{ref callee, ref args} => {
                let callee_ty = self.get_type(callee)?;
                let args_ty   = VecUtil::map(args, |arg| self.get_type(arg))?;
                if let Function{ref params_ty, ref return_ty} = callee_ty {
                    if params_ty.len() != args_ty.len() {
                        let msg = format!("Invalid number of args to {:?}",
                                          callee);
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
            If(ref e)  => {
                let cond_ty  = self.get_type(e.cond())?;
                let texpr_ty = self.get_type(e.texpr())?;
                let fexpr_ty = self.get_type(e.fexpr())?;
                ty_compare(&cond_ty, &Bool, "If condition")?;
                ty_compare(&texpr_ty, &fexpr_ty, "True expr and false expr")?;
                texpr_ty
            },
            ref expr   => { println!("NOTHANDLED\n{:?} not handled", expr);
                            unimplemented!() },
        };
        Ok(res)
    }

    fn tc_expr(&self, expr: &Expr) -> Result<Expr> {
        use ::hir::Expr::*;
        let res = match *expr {
            UnitLit    => UnitLit,
            I32Lit(n)  => I32Lit(n),
            BoolLit(b) => BoolLit(b),
            Var(ref v) => Var(v.clone()),
            App{ref callee, ref args} => {
                let callee = self.tc_expr(callee)?;
                let args = VecUtil::map(args, |arg| self.tc_expr(arg))?;
                App{callee: Box::new(callee), args}
            }
            If(ref e)  => {
                //FIXME: why do I need the self::
                let if_expr = self::If::new(self.tc_expr(e.cond())?,
                                            self.tc_expr(e.texpr())?,
                                            self.tc_expr(e.fexpr())?,
                                            self.get_type(e.fexpr())?);
                If(Box::new(if_expr))
            },
            ref expr   => { println!("NOTHANDLED\n{:?} not handled", expr);
                            unimplemented!() },
        };
        Ok(res)
    }
}
