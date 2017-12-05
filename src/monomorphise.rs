use hir::*;
use subst::Subst;
use std::rc::Rc;
use std::cell::RefCell;
use {Result,Error,VecUtil};
use scoped_map::ScopedMap;
use std::collections::HashMap;

pub struct Monomorphise {
    instances: Instances,
    curr_id: u32,
    toplevels: Vec<TopLevel>,
}

impl ::Pass for Monomorphise {
    type Input  = Vec<TopLevel>; 
    type Output = Vec<TopLevel>;

    fn run(mut self, toplevel_vec: Self::Input) -> Result<Self::Output> {
        for toplevel in toplevel_vec {
            self.mono_toplevel(toplevel)?;
        }
        Ok(self.toplevels)
    }
}

struct GenericExprCache {
    cache: ScopedMap<u32, (InstanceId,Expr)>,
    curr_id: u32
}

impl GenericExprCache {
    fn new() -> Self {
        Self{ cache: ScopedMap::new() }
    }

    fn begin_scope(&mut self) {
        self.cache.begin_scope();
    }

    fn end_scope(&mut self) {
        self.cache.end_scope();
    }

    fn insert(&mut self, id: InstanceId, ident: &Ident, expr: Expr) -> Result<()> {
        match self.cache.insert(ident.id(), (id, expr)) {
            Some(_) => {
                let msg = format!("{:?} is already marked as poly", ident);
                Err(Error::new(msg))
            }
            None => Ok(())
        }
    }

    fn get(&self, id: &Ident) -> Option<&(InstanceId, Expr)> {
        self.cache.get(&id.id())
    }
}


struct InstanceId(usize)
struct Instances {
    inner: Vec<ScopedMap<Vec<Type>, Ident>>,
}

impl Instances {
    fn new() -> Self {
        Self{ inner: vec![] }
    }

    fn add(&mut self) -> InstanceId {
        let size = inner.size();
        inner.push(ScopedMap::new());
        InstanceId(size)
    }

    fn begin_scope(&mut self, id: InstanceId) {
        unsafe{self.inner[id.0]}.begin_scope();
    }

    fn end_scope(&mut self, id: InstanceId) {
        unsafe{self.inner[id.0]}.end_scope();
    }

    fn insert(&mut self, id: InstanceId, ty: Vec<Type>, id: Ident) {
        unsafe{self.inner[id.0]}.insert(ty, id);
    }

    fn get(&mut self, id: InstanceId, ty: &Vec<Type>) -> Option<&Ident>{
        unsafe{self.inner[id.0]}.get(ty);
    }
}

impl Monomorphise {
    pub fn new() -> Self {
        Monomorphise{
            curr_id: 100000,
            toplevels: vec![],
            instances: Instances::new(),
        }
    }

    fn mono_toplevel(&mut self, toplevel: TopLevel) -> Result<()> {
        let mut cache = GenericExprCache::new();
        let mut tl = TopLevel::new(vec![]);
        let _ = VecUtil::mapt(toplevel.decls()
                              , |decl| self.mono_decl(decl, &mut cache, &mut tl));
        self.toplevels.push(tl);
        Ok(())
    }
    
    fn mono_decl(&mut self, decl: TopDecl
                 , cache: &mut GenericExprCache
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
                match self.mono_expr(&lam_expr, cache, toplevel)? {
                    Expr::Lam(lam) => toplevel.add_decl(Lam(lam.clone())),
                    _ => {}
                }
            }
        };
        Ok(())
    }

    fn new_id(&mut self) -> u32 {
        self.curr_id = self.curr_id + 1;
        self.curr_id
    }

    fn instantiate_ident(&mut self, ident: &Ident, subst: &Subst
                         , i: InstanceId) -> Ident
    {
        if ident.ty().is_monotype() {
            ident.clone()
        }
        else {
            let name  = format!("{}<{:?}>", ident.name(), subst);
            let ty    = subst.subst(ident.ty());
            let ident = Ident::new(Rc::new(name), ty, self.new_id());
            self.instances.insert(i, ident.id(), ident.clone());
            ident
        }
    }
    
    fn instantiate_fn_proto(&mut self, proto: &FnProto
                            , subst: &Subst, i: InstanceId) -> FnProto
    {
        let ident  = self.instantiate_ident(proto.ident(), subst, i);
        let params = proto.params().iter()
            .map(| ident| self.instantiate_ident(ident, subst, i))
            .collect();
        let ty     = ::types::ForAll::new(vec![], ident.ty().clone());
        FnProto::new(ident, params, ty)
    }

    fn instantiate(&mut self, expr: &Expr,subst: &Subst, i: InstanceId)
                   -> Result<Expr>
    {
        use hir::Expr::*;
        println!("HANDLE \n{:?}", expr);
        let res = match *expr {
            UnitLit    => UnitLit,
            I32Lit(n)  => I32Lit(n),
            BoolLit(b) => BoolLit(b),
            Var(ref id, ref tys) => {
                let id = match self.instances.get(i, &id.id()) {
                    Some(ident) => ident.clone(),
                    None        => id.clone(),
                };
                let id = self.instantiate_ident(&id, subst, i);
                Var(id, vec![])
            }
            Lam(ref lam) => {
                let proto = self.instantiate_fn_proto(lam.proto(), subst, i);
                let body  = self.instantiate(lam.body(), subst, i)?;
                let lam   = Lam(Rc::new(::hir::Lam::new(proto, body)));
                lam
            }
            ref expr  => panic!("Can not Instantiate {:?}", expr)
        };
        println!("Instantiation \n{:?}", res);

        Ok(res)
    }
    
    fn get_subst(&self, monotypes: &Vec<Type>, expr: &Expr) -> Result<Subst> {
        match *expr {
            Expr::Lam(ref lam) => Ok(lam.proto().ty().mk_subst(monotypes)),
            ref other => {
                let msg = format!("Cannot do subst for non lambda {:?}", expr);
                Err(Error::new(msg))
            }
        }
    }

    fn instantiate_var(&mut self, id: &Ident, monotypes: &Vec<Type>
                       , cache: &mut GenericExprCache
                       , toplevel: &mut TopLevel) -> Result<Expr>
    {
        match cache.get(&id) {
            None => {
                let var = Expr::Var((*id).clone(), vec![]);
                return Ok(var)
            },
            Some(&(i, ref gen_expr)) => {
                unsafe {
                    match self.instances.get(i, monotypes) {
                        Some(ident) => return Ok(Expr::Var(ident.clone(), vec![])),
                        None => {}
                    }
                }
                let expr  = {
                    let subst = self.get_subst(monotypes, &gen_expr)?;
                    self.instantiate(&gen_expr, &subst, i)?
                };
                match expr {
                    Expr::Lam(ref lam) => {
                        let id = lam.proto().ident().clone(); 
                        unsafe {
                            self.instantiations.get_unchecked_mut(i)
                                .insert(monotypes.clone(), id.clone());
                        }
                        toplevel.add_decl(TopDecl::Lam(lam.clone()));
                        Ok(Expr::Var(id.clone(), vec![]))
                    },
                    ref other => {
                        let msg = format!("Cannot do var subst for non lambda {:?}"
                                          , expr);
                        Err(Error::new(msg))
                    }
                }
            }
        }
    }
    
    fn mono_expr(&mut self, expr: &Expr, cache: &mut GenericExprCache
                 , toplevel: &mut TopLevel) -> Result<Expr> {
        use hir::Expr::*;
        let res = match *expr {
            UnitLit    => UnitLit,
            I32Lit(n)  => I32Lit(n),
            BoolLit(b) => BoolLit(b),
            Var(ref v, ref ty_vars) => {
                self.instantiate_var(v, ty_vars, cache, toplevel)?
            }
            App{ref callee, ref args} => {
                let callee = Box::new(self.mono_expr(callee, cache, toplevel)?);
                let args = VecUtil::map(args, |arg|
                                        self.mono_expr(arg, cache, toplevel))?;
                App{callee, args}
            }
            If(ref e)  => {
                let cond  = self.mono_expr(e.cond(),  cache, toplevel)?;
                let texpr = self.mono_expr(e.texpr(), cache, toplevel)?;
                let fexpr = self.mono_expr(e.fexpr(), cache, toplevel)?;
                let ty    = e.res_ty().clone();
                let ifexpr = self::If::new(cond, texpr, fexpr, ty);
                If(Box::new(ifexpr))
            }
            Lam(ref lam) => {
                if lam.proto().ty().is_monotype() {
                    let expr = self.mono_expr(lam.body(), cache, toplevel)?;
                    let lam  = ::hir::Lam::new(lam.proto().clone(), expr);
                    Lam(::std::rc::Rc::new(lam))
                }
                else {
                    let size = self.instantiations.len();
                    cache.insert(size, lam.ident(), Expr::Lam(lam.clone()))?;
                    self.instantiations.push(HashMap::new());
                    //Return a non Expr::Lam so result is not put in toplevel
                    UnitLit
                }
            }
            ref expr   => { println!("NOTHANDLED\n{:?} not handled", expr);
                            unimplemented!() },
        };
        Ok(res)
    }

}
