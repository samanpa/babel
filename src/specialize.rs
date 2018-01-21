use ::xir::*;
use ::types::Type;
use ::typing::subst::Subst;
use ::{Result,Vector};

pub struct Specialize {
    instances: Vec<Expr>
}

impl Default for Specialize {
    fn default() -> Self {
        Self::new()
    }
}

impl ::Pass for Specialize {
    type Input  = Vec<Module>;
    type Output = Vec<Module>;

    fn run(mut self, module_vec: Self::Input) -> Result<Self::Output> {
        let res = Vector::map(&module_vec, |module| self.mono_module(module))?;
        Ok(res)
    }
}

impl Specialize {
    pub fn new() -> Self {
        Specialize{ instances: vec![] }
    }

    fn mono_module(&mut self, module: &Module) -> Result<Module> {
        let decvec = Vector::map(module.decls(), |decl| self.mono_decl(decl))?;
        let mut res = vec![];
        for decls in decvec {
            for d in decls {
                res.push(d)
            }
        }
        Ok(Module::new(module.name().clone(), res))
    }

    fn mono_decl(&mut self, decl: &Decl) -> Result<Vec<Decl>> {
        use self::Decl::*;
        let res = match *decl {
            Extern(ref v, ref ty) => {
                vec![Extern(v.clone(), ty.clone())]
            }
            Let(ref id, ref expr) => {
                let mut sub  = Subst::new();
                let res_expr = mono(&expr, &mut sub)?;

                println!("{:?}", id);
                println!("\n{:?}", expr);
                println!("\n{:?}\n=================\n\n", res_expr);

                vec![Let(id.clone(), res_expr)]
            }
        };
        Ok(res)
    }
}

fn mono(expr: &Expr, sub: &mut Subst) -> Result<Expr>
{
    use ::xir;
    use self::Expr::*;
    let expr = match *expr {
        UnitLit     => UnitLit,
        I32Lit(n)   => I32Lit(n),
        BoolLit(b)  => BoolLit(b),
        Var(ref id) => Var(id.clone()),
        Lam(ref proto, ref body) => {
            let body = mono(body, sub)?;
            Lam(proto.clone(), Box::new(body))
        }
        If(ref e) => {
            let if_expr = xir::If::new(mono(e.cond(),  sub)?,
                                       mono(e.texpr(), sub)?,
                                       mono(e.fexpr(), sub)?);
            Expr::If(Box::new(if_expr))
        }
        App(ref callee, ref arg) => {
            let callee = mono(callee, sub)?;
            let arg    = mono(arg, sub)?;
            xir::Expr::App(Box::new(callee), Box::new(arg))
        }
        Let(ref exp) => {
            let exp = xir::Let::new(exp.id().clone(),
                                    mono(exp.bind(), sub)?,
                                    mono(exp.expr(), sub)?);
            Expr::Let(Box::new(exp))
        }
        TyLam(ref param, ref b) => {
            let body  = mono(b, sub)?;
            let tylam = TyLam(param.clone(), Box::new(body));
            tylam
        }
        TyApp(ref e, ref args) => {
            let args = args.iter()
                .map( |ty| sub.apply(ty) )
                .collect::<Vec<Type>>();
            match **e {
                TyLam(ref params, ref b) => {
                    for (tyvar, ty) in params.iter().zip(args.into_iter()) {
                        sub.bind(*tyvar, ty)
                    }
                    mono(b, sub)?
                }
                _ => {
                    let e = mono(e, sub)?;
                    TyApp(Box::new(e), args)
                }
            }
        }
    };
    Ok(expr)
}
    
