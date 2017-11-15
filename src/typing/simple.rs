use ::hir::*;
use ::{Result,Error,VecUtil};


pub struct SimpleTypeChecker {}

impl ::Pass for SimpleTypeChecker {
    type Input  = Vec<TopLevel>;
    type Output = Vec<TopLevel>;

    fn run(self, mut toplevel_vec: Self::Input) -> Result<Self::Output> {
        VecUtil::mapm(&mut toplevel_vec, |tl| self.tc_toplevel(tl))?;
        Ok(toplevel_vec)
    }
}

fn ty_compare<T: Into<String>>(t1: &Type, t2: &Type, msg: T) -> Result<()> {
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

    fn tc_toplevel(&self, toplevel: &mut TopLevel) -> Result<()> {
        VecUtil::mapm(toplevel.decls_mut(), |decl| self.tc_topdecl(decl))?;
        Ok(())
    }
    
    fn tc_topdecl(&self, decl: &mut TopDecl) -> Result<()> {
        use ::hir::TopDecl::*;
        let res = match *decl {
            Lam(ref mut lam)           => self.tc_lam(lam)?,
            Extern(ref proto, ref tys) => self.tc_proto(proto, tys)?,
        };
        Ok(res )
    }

    fn tc_proto(&self, proto: &Ident, tys: &Vec<Type>) -> Result<()> {
        //FIXME: Check if they are valid types currently not possible to
        //       have invalid types
        Ok(())
    }

    fn tc_lam(&self, lam: &mut Lam) -> Result<()> {
        let body_ty = self.get_type(lam.body())?;
        ty_compare(&body_ty, lam.return_ty()
                   , format!("lamba {:?}", lam.ident()))?;
        let mut expr = self.tc_expr(lam.body())?;
        *(lam.body_mut()) = expr;
        Ok(())
    }

    fn get_type(&self, expr: &Expr) -> Result<Type> {
        use ::hir::Type::*;
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
                        ty_compare(arg, param, format!("param type {}", i))?
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
