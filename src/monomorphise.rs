use hir::*;
use {Result,Error,VecUtil};
use scoped_map::ScopedMap;

pub struct Monomorphise {
    poly_funcs: ScopedMap<u32,Lam>,
    mono_funcs: ScopedMap<Vec<Type>,Lam>,
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

impl Monomorphise {
    pub fn new() -> Self {
        Monomorphise{
            toplevels: vec![],
            poly_funcs: ScopedMap::new(),
            mono_funcs: ScopedMap::new(),
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
                if proto.ty().ty_vars().len() > 0 {
                    let msg = format!("Extern func can not be polymorphic {:?}"
                                      , proto.ident());
                    return Err(Error::new(msg))
                }
                toplevel.add_decl(Extern(proto))
            },
            Lam(lam) => {
                if let Some(lam) = self.mono_lam(lam, toplevel)? {
                    toplevel.add_decl(Lam(lam))
                }                
            }
        };
        Ok(())
    }

    fn mono_lam(&mut self, lam: Lam
                , toplevel: &mut TopLevel) -> Result<Option<Lam>>
    {
        let res = match lam.proto().ty().ty_vars().len() {
            0 => {
                let expr = self.mono_expr(lam.body(), toplevel)?;
                let lam  = Lam::new(lam.proto().clone(), expr);
                Some(lam)
            }
            _ => {
                self.poly_funcs.insert(lam.ident().id(), lam);
                None
            }
        };
        Ok(res)
    }
    
    fn mono_expr(&mut self, expr: &Expr
                    , toplevel: &mut TopLevel) -> Result<Expr> {
        use hir::Expr::*;
        let res = match *expr {
            UnitLit    => UnitLit,
            I32Lit(n)  => I32Lit(n),
            BoolLit(b) => BoolLit(b),
            Var(ref v) => {
                Var(v.clone())
            }
            App{ref callee, ref args, ref ty_args} => {
                let callee = Box::new(self.mono_expr(callee, toplevel)?);
                let args = 
                    VecUtil::map(args, |arg| self.mono_expr(arg, toplevel))?;
                App{callee, args, ty_args: vec![]}
            }
            If(ref e)  => {
                let cond  = self.mono_expr(e.cond(), toplevel)?;
                let texpr = self.mono_expr(e.texpr(), toplevel)?;
                let fexpr = self.mono_expr(e.fexpr(), toplevel)?;
                let ty    = e.res_ty().clone();
                let ifexpr = self::If::new(cond, texpr, fexpr, ty);
                If(Box::new(ifexpr))
            },
            ref expr   => { println!("NOTHANDLED\n{:?} not handled", expr);
                            unimplemented!() },
        };
        Ok(res)
    }

}
