use ::xir::*;
use ::types::{Type};
use ::{Result,Vector};
use ::fresh_id;

use std::rc::Rc;

pub struct LambdaLift {
    scope: u32,
}

impl Default for LambdaLift {
    fn default() -> Self {
        Self::new()
    }
}

impl ::Pass for LambdaLift {
    type Input  = Vec<Module>;
    type Output = Vec<Module>;

    fn run(mut self, module_vec: Self::Input) -> Result<Self::Output> {
        let res = Vector::map(&module_vec, |module| self.lift_module(module))?;
        Ok(res)
    }
}

impl LambdaLift {
    pub fn new() -> Self {
        LambdaLift{scope: 0}
    }

    fn lift_module(&mut self, module: &Module) -> Result<Module> {
        let mut decls     = Vec::new();
        
        for decl in module.decls() {
            let decl = match *decl {
                Decl::Extern(ref symbol) => Decl::Extern(symbol.clone()),
                Decl::Let(ref bind)  => {
                    let bind = self.lift_bind(bind, &mut decls);
                    Decl::Let(bind)
                },
            };
            decls.push(decl)
        }

        for decl in decls.iter() {
            println!("{:?}", decl);
        }
        
        Ok(Module::new(module.name().clone(), decls))
    }
    
    fn lift_bind(&mut self, bind: &Bind, acc: &mut Vec<Decl>) -> Bind
    {
        self.scope += 1;
        let bind = match *bind {
            Bind::NonRec{ref symbol, ref expr} => {
                let expr = self.lift(expr, acc, true);
                match expr {
                    Expr::Lam(_, _, _) if self.scope > 1 => {
                        let fnnm = Rc::new(format!("@__anon_{}", fresh_id()));
                        let fnty = symbol.ty().clone();
                        let sym  = Symbol::new(fnnm, fnty, fresh_id());
                        let bind = Bind::non_rec(sym.clone(), expr);
                        acc.push(Decl::Let(bind));
                        Bind::non_rec(symbol.clone(), Expr::Var(sym))
                    }
                    _ =>  Bind::non_rec(symbol.clone(), expr)
                }                    
            }
        };
        self.scope -= 1;
        bind
    }

    fn lift(&mut self, expr: &Expr, acc: &mut Vec<Decl>, let_bound: bool)
            -> Expr
    {
        use ::xir;
        use self::Expr::*;
        match *expr {
            UnitLit     => UnitLit,
            I32Lit(n)   => I32Lit(n),
            BoolLit(b)  => BoolLit(b),
            Var(ref id) => Var(id.clone()),
            TyLam(_, _) => unimplemented!(),
            TyApp(_, _) => unimplemented!(),
            If(ref e) => {
                let if_expr = xir::If::new(self.lift(e.cond(),  acc, false),
                                           self.lift(e.texpr(), acc, false),
                                           self.lift(e.fexpr(), acc, false),
                                           e.ty().clone());
                Expr::If(Box::new(if_expr))
            }
            App(n, ref callee, ref arg) => {
                let callee = self.lift(callee, acc, false);
                let arg    = self.lift(arg, acc, false);
                App(n, Box::new(callee), Box::new(arg))
            }
            Let(ref exp) => {
                let bind = self.lift_bind(exp.bind(), acc);
                let expr = self.lift(exp.expr(), acc, false);
                let lexp = xir::Let::new(bind, expr);
                Let(Box::new(lexp))
            }
            Lam(ref proto, ref body, ref retty) => {
                let body  = self.lift(body, acc, false);
                let proto = proto.clone();
                let lam   = Lam(proto, Box::new(body), retty.clone());
                match let_bound {
                    true  => lam,
                    false => { //anonymous function
                        let fnty = retty.clone();
                        let fnnm = Rc::new(format!("@__anon_{}", fresh_id()));
                        let sym  = Symbol::new(fnnm, fnty, fresh_id());
                        let bind = Bind::non_rec(sym.clone(), lam);
                        acc.push(Decl::Let(bind));
                        Var(sym)
                    }
                }
            }
        }
    }
}
    
