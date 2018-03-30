use ::xir::*;
use ::types::{Type,TyVar};
use ::typing::subst::Subst;
use ::{Result,Vector,Error};
use ::fresh_id;
use scoped_map::ScopedMap;
use std::collections::HashMap;
use std::rc::Rc;

pub struct Specialize {}

struct Specializer {
    cache: Cache,
}

struct Cache {
    entries: ScopedMap<TermVar, Instances>,
}

impl Cache {
    fn new() -> Self {
        Self{ entries: ScopedMap::new() }
    }

    fn begin_scope(&mut self) {
        self.entries.begin_scope();
    }

    fn end_scope(&mut self) {
        self.entries.end_scope();
    }

    fn add_if_poly(&mut self, var: TermVar, expr: &Expr) -> bool {
        match *expr {
            Expr::TyLam(ref ty_args, _) if ty_args.len() > 0 => {
                self.entries.entry(var)
                    .or_insert_with( || Instances::new(ty_args.clone()) );
                true
            }
            _ => false
        }
    }
    
    fn add_instance(&mut self, var: &TermVar, sub: &mut Subst, args: Vec<Type>)
                    -> Result<TermVar>
    {
        match self.entries.get_mut(&var) {
            None => {
                Err(Error::new(format!("Could not find var {:?} -> {:?}"
                                       , var, args)))
            }
            Some(ref mut instance) => {
                let id = instance.add(var, sub, args);
                Ok(id)
            }
        }
    }

    fn is_poly(&self, var: &TermVar) -> bool {
        self.entries.get(var).is_some()
    }

    fn get(&self, id: &TermVar) -> Option<&Instances> {
        self.entries.get(id)
    }
}


struct Instances {
    tyvars: Vec<TyVar>,
    inner: HashMap<Vec<Type>, TermVar>,
}

impl Instances {
    fn new(tyvars: Vec<TyVar>) -> Self {
        Self{ tyvars, inner: HashMap::new() }
    }

    fn add(&mut self, var: &TermVar, sub: &mut Subst, args: Vec<Type>) -> TermVar {
        for (tyvar, ty) in self.tyvars.iter().zip(args.into_iter()) {
            sub.bind(*tyvar, ty.clone())
        }
        let args = self.tyvars.iter()
            .map( |ty| sub.apply(&Type::Var(*ty)) )
            .collect::<Vec<_>>();
        let var = self.inner.entry(args.clone())
            .or_insert_with( || {
                let name = format!("{}<{:?}>", var.name(), args);
                let ty = sub.apply(var.ty());
                TermVar::new(Rc::new(name), ty, fresh_id())
            });
        var.clone()
    }
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
        let res = Vector::mapt(module_vec, |module| self.mono_module(module))?;
        Ok(res)
    }
}

impl Specialize {
    pub fn new() -> Self {
        Specialize{}
    }

    fn mono_module(&mut self, module: Module) -> Result<Module> {
        let mut decls   = Vec::new();
        let mut spec    = Specializer{ cache: Cache::new()};
        let mut polytys = Vec::new();
        let mut monotys = Vec::new();
        let modname     = module.name().clone();
        for decl in module.take_decls() {
            match decl {
                e @ Decl::Extern( _) => decls.push(e),
                Decl::Let(id, expr)  => {
                    match spec.cache.add_if_poly(id.clone(), &expr) {
                        false => monotys.push((id, expr)),
                        true  => polytys.push((id, expr)),
                    }
                }
            }
        }

        for (id, expr) in monotys.into_iter().rev() {
            let mut sub = Subst::new();
            let expr = spec.specialize(&id, &expr, &mut sub, vec![])?;
            decls.push(Decl::Let(id, expr));
        }

        for (id, expr) in polytys.into_iter().rev() {
            let mut sub = Subst::new();
            for (id, expr) in spec.specialize_all(&id, &expr, &mut sub)? {
                decls.push(Decl::Let(id, expr));
            }
        }

        let decls = decls.into_iter().rev().collect();
        Ok(Module::new(modname, decls))
    }
}

impl Specializer
{
    fn specialize_all(&mut self, id: &TermVar, expr: &Expr
                       , sub: &mut Subst) -> Result<Vec<(TermVar, Expr)>>
    {
        let mut result = Vec::new();
        let instances = match self.cache.get(&id) {
            None => HashMap::new(),
            Some(ref instances) => instances.inner.clone()
        };
        for (tys, id) in instances {
            let tys = tys.iter().map( |ty| sub.apply(ty) ).collect();
            let mono_expr = self.specialize(&id, expr, sub, tys)?;
            result.push((id, mono_expr))
        };
        Ok(result)
    }

    fn specialize(&mut self, _id: &TermVar, expr: &Expr, sub: &mut Subst
              , args: Vec<Type>) -> Result<Expr>
    {
        //println!("==========\nSpecialize {:?} {:?}\n", id, args);
        //println!("{:?}\n", expr);

        self.cache.begin_scope();
        let expr    = self.run(&expr, sub, args)?;
        self.cache.end_scope();
        //println!("{:?}\n", expr);
        Ok(expr)
    }

    fn run(&mut self, expr: &Expr, sub: &mut Subst, args: Vec<Type>) -> Result<Expr>
    {
        use ::xir;
        use self::Expr::*;
        let expr = match *expr {
            UnitLit     => UnitLit,
            I32Lit(n)   => I32Lit(n),
            BoolLit(b)  => BoolLit(b),
            Lam(ref proto, ref body) => {
                let body  = self.run(body, sub, vec![])?;
                let proto = proto.iter()
                    .map( | id | id.with_ty(sub.apply(id.ty())) )
                    .collect();
                Lam(proto, Box::new(body))
            }
            If(ref e) => {
                let ty = sub.apply(e.ty());
                let if_expr = xir::If::new(self.run(e.cond(),  sub, vec![])?,
                                           self.run(e.texpr(), sub, vec![])?,
                                           self.run(e.fexpr(), sub, vec![])?,
                                           ty);
                Expr::If(Box::new(if_expr))
            }
            App(n, ref callee, ref arg) => {
                let callee = self.run(callee, sub, vec![])?;
                let arg    = self.run(arg, sub, vec![])?;
                xir::Expr::App(n, Box::new(callee), Box::new(arg))
            }
            TyLam(ref param, ref b) => {
                for (tyvar, ty) in param.iter().zip(args.into_iter()) {
                    sub.bind(*tyvar, ty)
                }
                let body = self.run(b, sub, vec![])?;
                body
            }
            TyApp(ref e, ref args) => {
                let args = args.iter()
                    .map( |ty| sub.apply(ty) )
                    .collect::<Vec<Type>>();
                self.run(e, sub, args)?
            }
            Var(ref id) => {
                //Check if the variable is monomorphic by looking in the
                //    polymorphic cache
                let id = match self.cache.is_poly(&id) {
                    false => id.with_ty(sub.apply(id.ty())),
                    true  => self.cache.add_instance(&id, sub, args)?
                };
                Var(id)
            }
            Let(ref exp) => {
                let let_id = exp.id();
                if self.cache.add_if_poly(let_id.clone(), exp.bind()) {
                    let mut let_expr = self.run(exp.expr(), sub, vec![])?;
                    let res  = self.specialize_all(&let_id, exp.bind(), sub)?;
                    for (id, bind) in res {
                        let exp  = xir::Let::new(id, bind, let_expr);
                        let_expr = Expr::Let(Box::new(exp))
                    }
                    let_expr
                }
                else {
                    // handle let x: 'a = ... Where 'a is monomorphic
                    let let_id   = let_id.with_ty(sub.apply(let_id.ty()));
                    let let_body = self.run(exp.expr(), sub, vec![])?;
                    let bind     = self.run(exp.bind(), sub, vec![])?;
                    let exp      = xir::Let::new(let_id, bind, let_body);
                    Expr::Let(Box::new(exp))
                }
            }
        };
        Ok(expr)
    }
}
    
