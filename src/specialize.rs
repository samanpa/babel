use ::xir::*;
use ::types::{Type,TyVar};
use ::typing::subst::Subst;
use ::{Result,Vector,Error};
use ::fresh_id;
use scoped_map::ScopedMap;
use std::collections::HashMap;
use std::rc::Rc;

pub struct Specialize {}

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

    //Go in reverse dependency order when specializing
    //    let id(x)     { x }
    //    let foo(f, x) { f(x) }
    //    let bar(x)    { foo(id, x)}
    //    let main()    { bar(2) }
    //  we specialize in order main, bar, foo, and id. So by the time we
    //     specialize any function we know all its type instantiations
    fn mono_module(&mut self, module: Module) -> Result<Module> {
        let mut spec      = Specializer::new();
        let mut decls     = Vec::new();
        let mut poly_exps = Vec::new();
        let mut mono_exps = Vec::new();
        let modname       = module.name().clone();
        
        for decl in module.take_decls() {
            match decl {
                e @ Decl::Extern( _) => decls.push(e),
                Decl::Let(b @ Bind::NonRec{..})  => {
                    match spec.add_if_poly(&b) {
                        false => mono_exps.push(b),
                        true  => poly_exps.push(b),
                    }
                }
            }
        }

        for bind in mono_exps.into_iter().rev() {
            let mut sub = Subst::new();
            let bind    = spec.process(&bind, &mut sub, vec![])?;
            decls.push(Decl::Let(bind));
        }

        for bind in poly_exps.into_iter().rev() {
            let mut sub = Subst::new();
            for bind in spec.process_all(&bind, &mut sub)? {
                decls.push(Decl::Let(bind));
            }
        }

        let decls = decls.into_iter().rev().collect();
        Ok(Module::new(modname, decls))
    }
}

struct Instances {
    tyvars: Vec<TyVar>,
    inner: HashMap<Vec<Type>, Symbol>,
}

impl Instances {
    fn new(tyvars: Vec<TyVar>) -> Self {
        Self{ tyvars, inner: HashMap::new() }
    }

    fn add(&mut self, var: &Symbol, sub: &mut Subst, args: Vec<Type>) -> Symbol {
        for (tyvar, ty) in self.tyvars.iter().zip(args.into_iter()) {
            sub.bind(*tyvar, ty)
        }
        let args = self.tyvars.iter()
            .map( |ty| sub.apply(&Type::Var(*ty)) )
            .collect::<Vec<_>>();
        let var = self.inner.entry(args.clone())
            .or_insert_with( || {
                let name = format!("{}<{:?}>", var.name(), args);
                let ty = sub.apply(var.ty());
                Symbol::new(Rc::new(name), ty, fresh_id())
            });
        var.clone()
    }
}


struct Specializer {
    entries: ScopedMap<Symbol, Instances>,
}

impl Specializer
{
    fn new() -> Self {
        Self{ entries: ScopedMap::new() }
    }

    fn begin_scope(&mut self) {
        self.entries.begin_scope();
    }

    fn end_scope(&mut self) {
        self.entries.end_scope();
    }

    fn add_if_poly(&mut self, b: &Bind) -> bool {
        use self::Bind::*;
        use self::Expr::TyLam;
        match *b {
            NonRec{ref symbol, ref expr} => {
                match *expr {
                    TyLam(ref tys, _) if tys.len() > 0 => {
                        self.entries.entry(symbol.clone())
                            .or_insert_with( || Instances::new(tys.clone()) );
                        true
                    },
                    _ => false,
                }
            }
        }
    }
    
    fn is_poly(&self, var: &Symbol) -> bool {
        self.entries.get(var).is_some()
    }

    fn add_instance(&mut self, var: &Symbol, sub: &mut Subst, args: Vec<Type>)
                    -> Result<Symbol>
    {
        match self.entries.get_mut(&var) {
            None => {
                Err(Error::new(format!("Could not find var {:?} -> {:?}"
                                       , var, args)))
            }
            Some(ref mut instances) => {
                let id = instances.add(var, sub, args);
                Ok(id)
            }
        }
    }

    fn get(&self, id: &Symbol) -> Option<&Instances> {
        self.entries.get(id)
    }
    
    fn process_all(&mut self, bind: &Bind, sub: &mut Subst) -> Result<Vec<Bind>>
    {
        let mut result = Vec::new();
        match *bind {
            Bind::NonRec{ref symbol, ref expr} => {
                let instances  = match self.get(&symbol) {
                    None => HashMap::new(),
                    Some(ref instances) => instances.inner.clone()
                };
                for (tys, symbol) in instances {
                    let tys  = tys.iter().map( |ty| sub.apply(ty) ).collect();
                    let spec   = self.spec(&symbol, expr, sub, tys)?;
                    let bind   = Bind::non_rec(symbol, spec);
                    result.push(bind);
                }
            }
        }
        Ok(result)
    }

    fn process(&mut self, bind: &Bind, sub: &mut Subst
               , args: Vec<Type>) -> Result<Bind>
    {
        let bind = match *bind {
            Bind::NonRec{ref symbol, ref expr} => {
                let spec = self.spec(symbol, expr, sub, args)?;
                // handle let symbol: 'a = ... Where 'a is monomorphic
                let symbol = symbol.with_ty(sub.apply(symbol.ty()));
                Bind::non_rec(symbol, spec)
            }
        };
        Ok(bind)
    }

    fn spec(&mut self, _sym: &Symbol, expr: &Expr, sub: &mut Subst
            , args: Vec<Type>) -> Result<Expr>
    {
        //println!("=============\nSpecialize {:?} \n{:?} {:?}", _sym, sub, args);
        //        println!("{:?}\n---------\n", expr);

        self.begin_scope();
        let expr   = self.run(expr, sub, args)?;
        self.end_scope();
        //println!("{:?}\n++++++++++++++\n\n", expr);
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
            Lam(ref proto, ref body, ref retty) => {
                let body  = self.run(body, sub, vec![])?;
                let proto = proto.iter()
                    .map( | id | id.with_ty(sub.apply(id.ty())) )
                    .collect();
                Lam(proto, Box::new(body), sub.apply(retty))
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
                let id = match self.is_poly(&id) {
                    false => id.with_ty(sub.apply(id.ty())),
                    true  => self.add_instance(&id, sub, args)?
                };
                Var(id)
            }
            Let(ref exp) => {
                let b        = exp.bind();
                if self.add_if_poly(b) {
                    let mut let_expr = self.run(exp.expr(), sub, vec![])?;
                    let res  = self.process_all(b, sub)?;
                    for bind in res {
                        let exp  = xir::Let::new(bind, let_expr);
                        let_expr = Expr::Let(Box::new(exp))
                    }
                    let_expr
                }
                else {
                    let let_body = self.run(exp.expr(), sub, vec![])?;
                    let bind = self.process(b, sub, vec![])?;
                    let exp  = xir::Let::new(bind, let_body);
                    Expr::Let(Box::new(exp))
                }
            }
        };
        Ok(expr)
    }
}
    
