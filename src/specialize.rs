use ::xir::*;
use ::types::Type;
use ::typing::subst::Subst;
use ::{Result,Vector,Error};
use scoped_map::ScopedMap;
use std::rc::Rc;
use std::collections::HashMap;

pub struct Specialize {}

struct Specializer {
    cache: Cache,
    mono: HashMap<TermVar, Expr>,
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

    fn insert_expr(&mut self, var: TermVar, expr: Expr) {
        self.entries.entry(var)
            .or_insert_with( || Instances::new(Rc::new(expr)) );
    }

    fn add_instance(&mut self, var: TermVar, ty: Type)
                    -> Result<(TermVar,Rc<Expr>, bool)>
    {
        match self.entries.get_mut(&var) {
            None => {
                Err(Error::new(format!("Could not find var {:?} -> {:?}"
                                       , var, ty)))
            }
            Some(ref mut instance) => {
                let (id, new) = instance.insert(&var, ty);
                Ok((id, instance.expr.clone(), new))
            }
        }
    }

    /*
    fn insert_instance(&mut self, ident: &TermVar, ty: &Vec<Type>
                       , instance: Ident) {
        self.entries.entry(ident.id())
            .or_insert_with( || Instances::new(None) )
            .insert(ty.clone(), instance);
    }

    fn get(&self, id: &TermVar) -> Option<&Instances> {
        self.entries.get(&id.id())
    }

    fn get_mut(&mut self, id: &TermVar) -> Option<&mut Instances> {
        self.entries.get_mut(&id.id())
    }
     */
}


struct Instances {
    expr: Rc<Expr>,
    inner: HashMap<Type, TermVar>,
}

impl Instances {
    fn new(expr: Rc<Expr>) -> Self {
        Self{ expr, inner: HashMap::new() }
    }
    fn insert(&mut self, var: &TermVar, ty: Type) -> (TermVar, bool) {
        let mut result = true;
        let var = self.inner.entry(ty.clone())
            .or_insert_with( || { result = true; var.with_ty(ty) } );
        (var.clone(), result)
    }
    fn insert_or_get(&mut self, ty: Type, var: TermVar) {
        self.inner.insert(ty, var);
    }

    fn get(&self, ty: &Type) -> Option<&TermVar> {
        self.inner.get(ty)
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
        let mut spec    = Specializer{ cache: Cache::new()
                                       , mono: HashMap::new()};
        let mut monotys = Vec::new();
        let modname     = module.name().clone();
        for decl in module.take_decls() {
            match decl {
                e @ Decl::Extern(_, _)  => decls.push(e),
                Decl::Let(id, expr)     => {
                    match id.ty().is_monotype() {
                        true  => monotys.push((id, expr)),
                        false => spec.cache.insert_expr(id, expr),
                    }
                }
            }
        }

        for (id, expr) in monotys {
            let mut sub = Subst::new();
            println!("{:?}\n", id);
            println!("{:?}\n", expr);

            spec.cache.begin_scope();
            let expr    = spec.run(&expr, &mut sub, vec![])?;
            spec.cache.end_scope();
            println!("{:?}\n========================================\n", expr);
            spec.mono.insert(id, expr);            
        }

        for (id, expr) in spec.mono {
            decls.push(Decl::Let(id, expr));

                
        }
      
        Ok(Module::new(modname, decls))
    }
}

impl Specializer
{
    fn run(&mut self, expr: &Expr, sub: &mut Subst, args: Vec<Type>) -> Result<Expr>
    {
        use ::xir;
        use self::Expr::*;
        let expr = match *expr {
            UnitLit     => UnitLit,
            I32Lit(n)   => I32Lit(n),
            BoolLit(b)  => BoolLit(b),
            Var(ref id) => {
                //Check if the variable is monomorphic.
                // Checking if the id.ty() has type variables is not sufficient
                // because a type variable can be non generic
                let ty = sub.apply(id.ty());
                match args.len() == 0 {
                    true  => Var(id.with_ty(ty)),
                    false => {
                        let id = id.clone();
                        let (id, exp, new) = self.cache.add_instance(id, ty)?;
                        if new {
                            let mono = self.run(&exp, sub, args)?;
                            self.mono.insert(id.clone(), mono);
                        }
                        Var(id)
                    }
                }
            }
            Lam(ref proto, ref body) => {
                let body = self.run(body, sub, vec![])?;
                Lam(proto.clone(), Box::new(body))
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
            Let(ref exp) => {
                if exp.id().ty().is_monotype() {
                    //self.cache.insert_expr(id, (*expr).clone());
                }
                let exp = xir::Let::new(exp.id().clone(),
                                        self.run(exp.bind(), sub, vec![])?,
                                        self.run(exp.expr(), sub, vec![])?);
                Expr::Let(Box::new(exp))
            }
            TyLam(ref param, ref b) => {
                for (tyvar, ty) in param.iter().zip(args.into_iter()) {
                    sub.bind(*tyvar, ty)
                }
                //FIXME:
                let body   = self.run(b, sub, vec![])?;
                body
            }
            TyApp(ref e, ref args) => {
                let args = args.iter()
                    .map( |ty| sub.apply(ty) )
                    .collect::<Vec<Type>>();
                let e = self.run(e, sub, args)?;
                e
            }
        };
        Ok(expr)
        }
}
    
