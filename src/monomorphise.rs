use hir::*;
use subst::Subst;
use std::rc::Rc;
use std::cell::RefCell;
use {Result,Error,VecUtil};
use scoped_map::ScopedMap;
use std::collections::HashMap;

pub struct Monomorphise {
    curr_id: u32,
    instantiations: ScopedMap<u32, InstanceMap>,
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

#[derive(Debug)]
struct InstanceMap {
    map: ::std::collections::HashMap<Vec<Type>, Ident>,
    generic_expr: Expr,
}

impl InstanceMap {
    fn new(expr: Expr) -> Self {
        InstanceMap{ generic_expr: expr
                     , map: ::std::collections::HashMap::new() }
    }
}


impl Monomorphise {
    pub fn new() -> Self {
        Monomorphise{
            curr_id: 100000,
            toplevels: vec![],
            instantiations: ScopedMap::new(),
        }
    }

    fn mono_toplevel(&mut self, toplevel: TopLevel) -> Result<()> {
        let mut tl = TopLevel::new(vec![]);
        let _ = VecUtil::mapt(toplevel.decls()
                              , |decl| self.mono_decl(decl, &mut tl));
        self.toplevels.push(tl);
        Ok(())
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

    fn record_poly(&mut self, id: &Ident, expr: Expr) -> Result<()> {
        match self.instantiations.insert(id.id(), InstanceMap::new(expr)) {
            Some(_) => {
                let msg = format!("{:?} is already marked as poly", id);
                Err(Error::new(msg))
            }
            None => Ok(())
        }
    }

    fn new_id(&mut self) -> u32 {
        self.curr_id = self.curr_id + 1;
        self.curr_id
    }

    fn instantiate_ident(&mut self, ident: &Ident, subst: &Subst
                         , env: &mut HashMap<u32, Ident>) -> Ident {
        if ident.ty().is_monotype() {
            ident.clone()
        }
        else {
            let name  = format!("{}<{:?}>", ident.name(), subst);
            let ty    = subst.subst(ident.ty());
            let ident = Ident::new(Rc::new(name), ty, self.new_id());
            env.insert(ident.id(), ident.clone());
            ident
        }        
    }
    
    fn instantiate_fn_proto(&mut self, proto: &FnProto
                            , subst: &Subst
                            , env: &mut HashMap<u32, Ident>) -> FnProto
    {
        let ident  = self.instantiate_ident(proto.ident(), subst, env);
        let params = proto.params().iter()
            .map(| ident| self.instantiate_ident(ident, subst, env))
            .collect();
        let ty     = ::types::ForAll::new(vec![], ident.ty().clone());
        FnProto::new(ident, params, ty)
    }

    fn instantiate(&mut self, expr: &Expr,subst: &Subst
                   , env: &mut HashMap<u32, Ident>) -> Result<Expr>
    {
        use hir::Expr::*;
        let res = match *expr {
            UnitLit    => UnitLit,
            I32Lit(n)  => I32Lit(n),
            BoolLit(b) => BoolLit(b),
            Var(ref id, ref tys) => {
                match env.get(&id.id()) {
                    Some(ident) => Var(ident.clone(), vec![]),
                    None        => Var(id.clone(), tys.clone()),
                }
            }
            Lam(ref lam) => {
                let proto = self.instantiate_fn_proto(lam.proto(), subst, env);
                let body  = self.instantiate(lam.body(), subst, env)?;
                let lam   = Lam(Rc::new(::hir::Lam::new(proto, body)));
                lam
            }
            ref expr  => panic!("Can not Instantiate {:?}", expr)
        };
        println!("Instantiation {:?}\n{:?} ", expr, res);
        Ok(res)
    }

    fn get_subst(monotypes: &Vec<Type>, expr: &Expr) -> Result<Subst> {
        match *expr {
            Expr::Lam(ref lam) => Ok(lam.proto().ty().mk_subst(monotypes)),
            ref other => {
                let msg = format!("Cannot do subst for non lambda {:?}", expr);
                Err(Error::new(msg))
            }
        }
    }

    fn instantiate_var(&mut self, id: &Ident, monotypes: &Vec<Type>
                       , toplevel: &mut TopLevel) -> Result<Expr>
    {
        match self.instantiations.get_mut(&id.id()) {
            None => {
                let var = Expr::Var((*id).clone(), vec![]);
                return Ok(var)
            },
            Some(ref mut inst_map) => {
                match inst_map.map.get(monotypes) {
                    Some(ident) => return Ok(Expr::Var(id.clone(), vec![])),
                    None => {}
                }
                let mut env = HashMap::new();
                let expr  = {
                    let gen_expr = &inst_map.generic_expr;
                    let subst = Monomorphise::get_subst(monotypes, &gen_expr)?;
                    self.instantiate(&gen_expr, &subst, &mut env)?
                };
                match expr {
                    Expr::Lam(ref lam) => {
                        let id = lam.proto().ident().clone(); 
                        inst_map.map.insert(monotypes.clone(), id.clone());
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
    
    fn mono_expr(&mut self
                 , expr: &Expr
                 , toplevel: &mut TopLevel) -> Result<Expr> {
        use hir::Expr::*;
        let res = match *expr {
            UnitLit    => UnitLit,
            I32Lit(n)  => I32Lit(n),
            BoolLit(b) => BoolLit(b),
            Var(ref v, ref ty_vars) => {
                self.instantiate_var(v, ty_vars, toplevel)?
            }
            App{ref callee, ref args} => {
                let callee = Box::new(self.mono_expr(callee, toplevel)?);
                let args = 
                    VecUtil::map(args, |arg| self.mono_expr(arg, toplevel))?;
                App{callee, args}
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
                    self.instantiations.begin_scope();
                    let expr = self.mono_expr(lam.body(), toplevel)?;
                    let lam  = ::hir::Lam::new(lam.proto().clone(), expr);
                    self.instantiations.end_scope();
                    Lam(::std::rc::Rc::new(lam))
                }
                else {
                    self.record_poly(lam.ident(), Expr::Lam(lam.clone()))?;
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
