use crate::fresh_id;
use crate::scoped_map::ScopedMap;
use crate::xir::*;
use crate::{Result, Vector};

use std::rc::Rc;

pub struct LambdaLift {
    map: ScopedMap<u32, Symbol>,
}

impl Default for LambdaLift {
    fn default() -> Self {
        Self::new()
    }
}

impl crate::Pass for LambdaLift {
    type Input = Vec<Module>;
    type Output = Vec<Module>;

    fn run(mut self, module_vec: Self::Input) -> Result<Self::Output> {
        let res = Vector::map(&module_vec, |module| self.lift_module(module))?;
        Ok(res)
    }
}

impl LambdaLift {
    pub fn new() -> Self {
        LambdaLift {
            map: ScopedMap::new(),
        }
    }

    fn lift_module(&mut self, module: &Module) -> Result<Module> {
        let mut decls = Vec::new();

        for decl in module.decls() {
            let decl = match *decl {
                Decl::Extern(ref symbol) => Decl::Extern(symbol.clone()),
                Decl::Let(ref bind) => {
                    let bind = bind
                        .iter()
                        .map(|bind| self.lift_bind(bind, &mut decls))
                        .collect();
                    Decl::Let(bind)
                }
            };
            decls.push(decl)
        }

        /*
        for decl in decls.iter() {
            //println!("{:?}\n", decl);
        }
        */

        Ok(Module::new(module.name().clone(), decls))
    }

    fn lift_bind(&mut self, bind: &Bind, acc: &mut Vec<Decl>) -> Bind {
        self.map.begin_scope();
        let symbol = bind.symbol();
        let expr = self.lift(bind.expr(), acc, true);
        let bind = match expr {
            Expr::Lam(_, _, _) if self.map.scope() > 1 => {
                let fnid = fresh_id();
                let fnnm = Rc::new(format!("@__fnanon_{}", fnid));
                let fnty = symbol.ty().clone();
                let sym = Symbol::new(fnnm, fnty, fnid);
                let bind = Bind::new(symbol.clone(), expr);
                self.map.insert(symbol.id(), sym.clone());
                acc.push(Decl::Let(vec![bind]));
                Bind::new(sym, Expr::Var(symbol.clone()))
            }
            _ => Bind::new(symbol.clone(), expr),
        };
        self.map.end_scope();
        bind
    }

    fn lift(&mut self, expr: &Expr, acc: &mut Vec<Decl>, let_bound: bool) -> Expr {
        use self::Expr::*;
        use crate::xir;
        match *expr {
            UnitLit => UnitLit,
            I32Lit(n) => I32Lit(n),
            BoolLit(b) => BoolLit(b),
            Var(ref id) => {
                let sym = match self.map.get(&id.id()) {
                    Some(sym) => sym.clone(),
                    None => id.clone(),
                };
                Var(sym)
            }
            TyLam(ref t, ref e) => {
                let e = self.lift(e, acc, false);
                TyLam(t.clone(), Box::new(e))
            }
            TyApp(ref e, ref t) => {
                let e = self.lift(e, acc, false);
                TyApp(Box::new(e), t.clone())
            }
            If(ref e) => {
                let if_expr = xir::If::new(
                    self.lift(e.cond(), acc, false),
                    self.lift(e.texpr(), acc, false),
                    self.lift(e.fexpr(), acc, false),
                    e.ty().clone(),
                );
                Expr::If(Box::new(if_expr))
            }
            App(ref ty, ref callee, ref args) => {
                let callee = self.lift(callee, acc, false);
                let args = args
                    .iter()
                    .map(|arg| self.lift(arg, acc, false))
                    .collect::<Vec<_>>();
                App(ty.clone(), Box::new(callee), args)
            }
            Let(ref exp) => {
                let bind = self.lift_bind(exp.bind(), acc);
                let expr = self.lift(exp.expr(), acc, false);
                let lexp = xir::Let::new(bind, expr);
                Let(Box::new(lexp))
            }
            Lam(ref proto, ref body, ref retty) => {
                let body = self.lift(body, acc, false);
                let proto = proto.clone();
                let lam = Lam(proto, Box::new(body), retty.clone());
                match let_bound {
                    true => lam,
                    false => {
                        //anonymous function
                        let fnty = retty.clone();
                        let fnnm = Rc::new(format!("@__anon_{}", fresh_id()));
                        let sym = Symbol::new(fnnm, fnty, fresh_id());
                        let bind = Bind::new(sym.clone(), lam);
                        acc.push(Decl::Let(vec![bind]));
                        Var(sym)
                    }
                }
            }
        }
    }
}
