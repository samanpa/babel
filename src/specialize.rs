use ::xir::*;
use ::types::Type;
use ::typing::subst::Subst;
use ::{Result,Vector,Error};
use scoped_map::ScopedMap;
use std::collections::HashMap;

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

    fn insert_expr(&mut self, var: TermVar) {
        self.entries.entry(var)
            .or_insert_with( || Instances::new() );
    }

    fn add_instance(&mut self, var: TermVar, ty: Type, args: Vec<Type>)
                    -> Result<TermVar>
    {
        match self.entries.get_mut(&var) {
            None => {
                Err(Error::new(format!("Could not find var {:?} -> {:?}"
                                       , var, ty)))
            }
            Some(ref mut instance) => {
                let id = instance.add(&var, ty, args);
                Ok(id)
            }
        }
    }

    fn get(&self, id: &TermVar) -> Option<&Instances> {
        self.entries.get(id)
    }
}


struct Instances {
    inner: HashMap<Vec<Type>, TermVar>,
}

impl Instances {
    fn new() -> Self {
        Self{ inner: HashMap::new() }
    }

    fn add(&mut self, var: &TermVar, ty: Type, args: Vec<Type>) -> TermVar {
        let var = self.inner.entry(args.clone())
            .or_insert_with( || var.with_ty(ty) );
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
        let mut mono    = Vec::new();
        let mut poly    = Vec::new();
        let mut monotys = Vec::new();
        let modname     = module.name().clone();
        for decl in module.take_decls() {
            match decl {
                e @ Decl::Extern(_, _)  => decls.push(e),
                Decl::Let(id, expr)     => {
                    match id.ty().is_monotype() {
                        true  => monotys.push((id, expr)),
                        false => {
                            spec.cache.insert_expr(id.clone());
                            poly.push((id, expr));
                        }
                    }
                }
            }
        }

        for (id, expr) in monotys {
            let mut sub = Subst::new();
            spec.cache.begin_scope();
            let expr = spec.do_run(&expr, &mut sub, vec![])?;
            mono.push((id, expr));
            spec.cache.end_scope();
        }

        for (id, expr) in poly.into_iter().rev() {
            println!("========\n{:?}", id);
            let mut sub = Subst::new();
            spec.cache.begin_scope();
            for (id, expr) in spec.run_poly(&id, &expr, &mut sub, vec![])? {
                decls.push(Decl::Let(id, expr));
            }
            spec.cache.end_scope();
        }
        
        Ok(Module::new(modname, decls))
    }
}

impl Specializer
{
    fn run_poly(&mut self, id: &TermVar, expr: &Expr, sub: &mut Subst
                , args: Vec<Type>) -> Result<Vec<(TermVar, Expr)>>
    {
        let mut result = Vec::new();
        let instances = match self.cache.get(&id) {
            None => HashMap::new(),
            Some(ref instances) => instances.inner.clone()
        };
        for (tys, id) in &instances {
            println!("Specialize {:?} === {:?}\n", id, tys);
            let tys = tys.iter().map( |ty| sub.apply(ty) ).collect();
            let mono_expr = self.do_run(expr, sub, tys)?;
            result.push((id.clone(), mono_expr))
        };
        Ok(result)
    }

    fn do_run(&mut self, expr: &Expr, sub: &mut Subst, args: Vec<Type>)
              -> Result<Expr>
    {
        println!("{:?}\n", expr);

        let expr    = self.run(&expr, sub, args)?;
        println!("{:?}\n========================================\n", expr);
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
                let if_expr = xir::If::new(self.run(e.cond(),  sub, vec![])?,
                                           self.run(e.texpr(), sub, vec![])?,
                                           self.run(e.fexpr(), sub, vec![])?);
                Expr::If(Box::new(if_expr))
            }
            App(ref callee, ref arg) => {
                let callee = self.run(callee, sub, vec![])?;
                let arg    = self.run(arg, sub, vec![])?;
                xir::Expr::App(Box::new(callee), Box::new(arg))
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
                //Check if the variable is monomorphic.
                //Checking if the id.ty() has type variables is not 
                //   sufficient because a type variable can be non generic
                let ty = sub.apply(id.ty());
                match args.len() == 0 {
                    true  => Var(id.with_ty(ty)),
                    false => {
                        let id = id.clone();
                        let id = self.cache.add_instance(id, ty, args)?;
                        Var(id)
                    }
                }
            }
            Let(ref exp) => {
                let ty = sub.apply(exp.id().ty());
                let id = exp.id().with_ty(ty);

                match id.ty().is_monotype() {
                    true  => {
                        let let_body = self.run(exp.expr(), sub, vec![])?;
                        let bind = self.run(exp.bind(), sub, vec![])?;
                        let exp  = xir::Let::new(id, bind, let_body);
                        Expr::Let(Box::new(exp))
                    }
                    false => {
                        self.cache.insert_expr(id);
                        let id   = exp.id();
                        let bind = exp.bind();
                        let res  = self.run_poly(id, bind, sub, args)?;
                        let mut let_expr = self.run(exp.expr(), sub, vec![])?;
                        for (id, bind) in res {
                            let exp  = xir::Let::new(id, bind, let_expr);
                            let_expr = Expr::Let(Box::new(exp))
                        }
                        let_expr
                    }
                }
            }
        };
        Ok(expr)
    }
}
    
