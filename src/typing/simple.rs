use ::ast::*;
use ::{Result,Error,VecUtil};
use std::collections::HashMap;


type VarTy = ::rename::Var;

pub struct SimpleTypeChecker {
    type_env: HashMap<u32, Type>,
}

impl ::Pass for SimpleTypeChecker {
    type Input  = Vec<TopLevel<VarTy>>;
    type Output = Vec<TopLevel<VarTy>>;

    fn run(&mut self, toplevel_vec: Self::Input) -> Result<Self::Output> {
        let res = VecUtil::map(&toplevel_vec, |tl| self.tc_toplevel(tl))?;
        Ok(res)
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
        SimpleTypeChecker{type_env: HashMap::new()}
    }

    fn tc_toplevel(&mut self, toplevel: &TopLevel<VarTy>) -> Result<TopLevel<VarTy>> {
        let res = VecUtil::map(toplevel.decls(), |decl| self.tc_topdecl(decl))?;
        Ok(TopLevel::new(res))
    }
    
    fn tc_topdecl(&mut self, decl: &TopDecl<VarTy>) -> Result<TopDecl<VarTy>> {
        use ::ast::TopDecl::*;
        let res = match *decl {
            Lam(ref lam)      => Lam(Box::new(self.tc_lam(&**lam)?)),
            Extern(ref proto) => Extern(self.tc_proto(proto)),
            Use{..} => unimplemented!()
        };
        Ok(res )
    }

    fn add_ident(&mut self, var: &VarTy) {
        self.type_env.insert(var.id(), var.ty().clone());
    }

    fn tc_params(params: &Vec<Param<VarTy>>) -> Vec<Param<VarTy>>{
        params.iter()
            .map(|param| Param::new(param.name().clone(), param.ty().clone()))
            .collect()
    }

    fn tc_proto(&mut self, proto: &FnProto<VarTy>) -> FnProto<VarTy> {
        let var = proto.name().clone();
        let params = Self::tc_params(proto.params());
        let proto = FnProto::new(var, params, proto.return_ty().clone());
        proto
    }

    fn tc_lam(&mut self, lam: &Lam<VarTy>) -> Result<Lam<VarTy>> {
        let body_ty = self.get_type(lam.body())?;
        ty_compare(&body_ty, lam.proto().return_ty()
                   , format!("lamba {:?}", lam.proto().name()))?;

        let func_nm = lam.name().clone();
        let params = Self::tc_params(lam.params());
        let body = self.tc_expr(lam.body())?;
        let lam = Lam::new(func_nm, params, lam.return_ty().clone(), body);


        Ok(lam)

        
    }

    fn get_type(&mut self, expr: &Expr<VarTy>) -> Result<Type> {
        use ::ast::Type::*;
        use ::ast::BaseType::*;
        use ::ast::Expr::*;
        let res = match *expr {
            UnitLit    => BaseType(Unit),
            I32Lit(n)  => BaseType(I32),
            BoolLit(b) => BaseType(Bool),
            Var(ref v) => v.ty().clone(),
            App{ref callee, ref args} => {
                let callee_ty = self.get_type(callee)?;
                let args_ty   = VecUtil::map(args, |arg| self.get_type(arg))?;
                if let FunctionType{ref params_ty, ref return_ty} = callee_ty {
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
                ty_compare(&cond_ty, &BaseType(Bool), "If condition")?;
                ty_compare(&texpr_ty, &fexpr_ty, "True expr and false expr")?;
                texpr_ty
            },
            ref expr   => { println!("NOTHANDLED\n{:?} not handled", expr);
                            unimplemented!() },
        };
        Ok(res)
    }

    fn tc_expr(&mut self, expr: &Expr<VarTy>) -> Result<Expr<VarTy>> {
        use ::ast::Expr::*;
        let ty = self.get_type(expr)?;
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
                                            Some(ty));
                If(Box::new(if_expr))
            },
            ref expr   => { println!("NOTHANDLED\n{:?} not handled", expr);
                            unimplemented!() },
        };
        Ok(res)
    }
}