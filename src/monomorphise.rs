use hir::*;
use subst::Subst;
use std::rc::Rc;
use {Result,Error,Vector};
use scoped_map::ScopedMap;
use std::collections::HashMap;
use fresh_id;

pub struct Monomorphise {
    cache: Cache,
}

impl ::Pass for Monomorphise {
    type Input  = Vec<TopLevel>; 
    type Output = Vec<TopLevel>;

    fn run(mut self, toplevel_vec: Self::Input) -> Result<Self::Output> {
        let toplevels = Vector::mapt(toplevel_vec
                                     , |tl| self.mono_toplevel(tl))?;
        Ok(toplevels)
    }
}

struct Cache {
    entries: ScopedMap<u32, Instances>,
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

    fn insert_expr(&mut self, ident: &Ident, expr: Expr) {
        self.entries.entry(ident.id())
            .or_insert_with( || Instances::new(Some(expr)) );
    }

    fn insert_instance(&mut self, ident: &Ident, ty: &Vec<Type>
                       , instance: Ident) {
        self.entries.entry(ident.id())
            .or_insert_with( || Instances::new(None) )
            .insert(ty.clone(), instance);
    }

    fn get(&self, id: &Ident) -> Option<&Instances> {
        self.entries.get(&id.id())
    }

    fn get_mut(&mut self, id: &Ident) -> Option<&mut Instances> {
        self.entries.get_mut(&id.id())
    }
}

struct Instances {
    expr: Option<Rc<Expr>>,
    inner: HashMap<Vec<Type>, Ident>,
}

impl Instances {
    fn new(expr: Option<Expr>) -> Self {
        let expr = expr.map( |expr| Rc::new(expr) );
        Self{ expr, inner: HashMap::new() }
    }
    fn insert(&mut self, ty: Vec<Type>, ident: Ident) {
        self.inner.insert(ty, ident);
    }

    fn get(&self, ty: &Vec<Type>) -> Option<&Ident> {
        self.inner.get(ty)
    }
}

impl Monomorphise {
    pub fn new() -> Self {
        Monomorphise{ cache: Cache::new() }
    }

    fn mono_toplevel(&mut self, toplevel: TopLevel) -> Result<TopLevel> {
        let mut tl = TopLevel::new(vec![]);
        Vector::mapt(toplevel.decls(), |decl| self.mono_decl(decl, &mut tl))?;
        Ok(tl)
    }
    
    fn mono_decl(&mut self, decl: TopDecl
                 , toplevel: &mut TopLevel) -> Result<()> {
        use hir::TopDecl::*;
        match decl {
            Extern(proto) => {
                if !proto.ty().is_monotype() {
                    let msg = format!("Extern func can not be polymorphic {:?}"
                                      , proto.ident());
                    return Err(Error::new(msg))
                }
                toplevel.add_decl(Extern(proto))
            },
            Lam(lam) => {
                let lam_expr = Expr::Lam(lam.clone());
                match self.mono_expr(&lam_expr, toplevel)? {
                    Expr::Lam(lam) => toplevel.add_decl(Lam(lam.clone())),
                    _ => {}
                }
            }
        };
        Ok(())
    }

    fn instantiate_ident(&mut self, ident: &Ident, subst: &Subst) -> Ident
    {
        if ident.ty().is_monotype() {
            ident.clone()
        }
        else {
            if let Some(ref inst) = self.cache.get(ident) {
                if let Some(ref ident) = inst.get(subst.range()) {
                    return (*ident).clone();
                }
            }
            let name   = format!("{}<{:?}>", ident.name(), subst.range());
            let ty     = subst.apply(ident.ty());
            let ident_ = Ident::new(Rc::new(name), ty, fresh_id());
            self.cache.insert_instance(ident, subst.range(), ident_.clone());
            ident_
        }
    }
    
    fn instantiate_fn_proto(&mut self, proto: &FnProto
                            , subst: &Subst) -> FnProto
    {
        let ident  = self.instantiate_ident(proto.ident(), subst);
        let params = proto.params().iter()
            .map(| ident| self.instantiate_ident(ident, subst))
            .collect();
        let ty     = ::types::ForAll::new(vec![], ident.ty().clone());
        FnProto::new(ident, params, ty)
    }

    fn instantiate(&mut self, expr: &Expr,subst: &Subst) -> Result<Expr>
    {
        use hir::Expr::*;
        let res = match *expr {
            UnitLit    => UnitLit,
            I32Lit(n)  => I32Lit(n),
            BoolLit(b) => BoolLit(b),
            Var(ref id, _) => {
                let id = match self.cache.get(id) {
                    None => id.clone(),
                    Some(instance) => 
                        match instance.get(subst.range()) {
                            Some(id) => id.clone(),
                            None     => id.clone()
                        }
                };
                let id = self.instantiate_ident(&id, subst);
                Var(id, vec![])
            }
            App(ref callee, ref args) => {
                let callee = Box::new(self.instantiate(callee, subst)?);
                let args   = Vector::map(args, |arg|
                                         self.instantiate(arg, subst))?;
                App(callee, args)
            }
            If(ref e)  => {
                let cond  = self.instantiate(e.cond(),  subst)?;
                let texpr = self.instantiate(e.texpr(), subst)?;
                let fexpr = self.instantiate(e.fexpr(), subst)?;
                let ty    = e.res_ty().clone(); //FIXME
                let ifexpr = self::If::new(cond, texpr, fexpr, ty);
                If(Box::new(ifexpr))
            }
            Lam(ref lam) => {
                self.cache.begin_scope();
                let proto = self.instantiate_fn_proto(lam.proto(), subst);
                let body  = self.instantiate(lam.body(), subst)?;
                let lam   = Lam(Rc::new(::hir::Lam::new(proto, body)));
                self.cache.end_scope();
                lam
            }
        };

        Ok(res)
    }
    
    fn get_subst(&self, monotypes: &Vec<Type>, expr: &Expr) -> Result<Subst> {
        match *expr {
            Expr::Lam(ref lam) => Ok(lam.proto().ty().mk_subst(monotypes)),
            _ => {
                let msg = format!("Cannot do subst for non lambda {:?}", expr);
                Err(Error::new(msg))
            }
        }
    }

    fn instantiate_var(&mut self, id: &Ident, monotypes: &Vec<Type>
                       , toplevel: &mut TopLevel) -> Result<Expr>
    {
        let rc_expr = match self.cache.get(&id) {
            None => {
                //Monomorphic
                let var = Expr::Var((*id).clone(), vec![]);
                return Ok(var)
            },
            Some(ref instances) => {
                match instances.get(monotypes) {
                    Some(ident) => return Ok(Expr::Var(ident.clone(), vec![])),
                    None => {}
                }
                match instances.expr {
                    Some(ref rc_expr) => rc_expr.clone(),
                    None => panic!("Can not instantiate non expr")
                }
            }
        };
        
        let subst = self.get_subst(monotypes, &*rc_expr)?;
        let expr  = self.instantiate(&*rc_expr, &subst)?;
        match expr {
            Expr::Lam(ref lam) => {
                let instance = lam.proto().ident().clone();
                self.cache.get_mut(id)
                    .unwrap()
                    .insert(subst.range().clone(), instance.clone());
                toplevel.add_decl(TopDecl::Lam(lam.clone()));
                Ok(Expr::Var(instance, vec![]))
            },
            _ => {
                let msg = format!("Cannot do var subst for non lambda {:?}"
                                  , expr);
                Err(Error::new(msg))
            }
        }
    }
    
    fn mono_expr(&mut self, expr: &Expr
                 , toplevel: &mut TopLevel) -> Result<Expr> {
        use hir::Expr::*;
        let res = match *expr {
            UnitLit    => UnitLit,
            I32Lit(n)  => I32Lit(n),
            BoolLit(b) => BoolLit(b),
            Var(ref v, ref ty_vars) => {
                self.instantiate_var(v, ty_vars, toplevel)?
            }
            App(ref callee, ref args) => {
                let callee = Box::new(self.mono_expr(callee, toplevel)?);
                let args = Vector::map(args, |arg|
                                        self.mono_expr(arg, toplevel))?;
                App(callee, args)
            }
            If(ref e)  => {
                let cond  = self.mono_expr(e.cond(), toplevel)?;
                let texpr = self.mono_expr(e.texpr(), toplevel)?;
                let fexpr = self.mono_expr(e.fexpr(), toplevel)?;
                let ty    = e.res_ty().clone();
                let ifexpr = self::If::new(cond, texpr, fexpr, ty);
                If(Box::new(ifexpr))
            }
            Lam(ref lam) => {
                if lam.proto().ty().is_monotype() {
                    let expr = self.mono_expr(lam.body(), toplevel)?;
                    let lam  = ::hir::Lam::new(lam.proto().clone(), expr);
                    Lam(::std::rc::Rc::new(lam))
                }
                else {
                    self.cache.insert_expr(lam.ident()
                                           , Expr::Lam(lam.clone()));
                    //Return a non Expr::Lam so result is not put in toplevel
                    UnitLit
                }
            }
        };
        Ok(res)
    }
}
