use ast::*;
use {Result,Error};
use scoped_map::ScopedMap;

#[derive(Clone)]
pub struct Name {
    name: Rc<String>,
    ty:   Type,
    id:   u32,
}

pub struct Rename {
    count: u32,
}

impl ::Pass for Rename {
    type Input  = Vec<TopLevel<String>>; //A list of parsed files
    type Output = Vec<TopLevel<Name>>;

    fn run(&mut self, toplevel_vec: Self::Input) -> Result<Self::Output> {
        let mut names = ScopedMap::new();
        for toplevel in &toplevel_vec {
            self.trans_toplevel(toplevel, &mut names)?;
        }
        Ok(names)
    }
}

impl Rename {
    fn new() -> Self {
        Rename{count: 0}
    }

    fn rename_toplevel(&mut self
                       , toplevel: &TopLevel<String>
                       , names: &mut ScopedMap<u32, Name>) -> Result<()> {
        let decls = Vec::new();
        for decl in toplevel.decls() {
            decls.push(self.rename_topdecl(decl, names));
        }
        Ok(())
    }
    
    fn rename_topdecl(&mut self
                      , decl: &TopDecl<String>
                      , names: &mut ScopedMap<u32, Name>) -> Result<TopDecl<Name>> {
        use TopDecl::*;
        let res = match *decl {
            Extern{ref name, ref ty} => {
                let new_name = Name{ name: Rc::new(self.name.clone()),
                                 ty: ty.clone(),
                                 id: self.count };
                if let Some(ref nm) = names.insert(name, new_name) {
                    return Error::new(format!("Name {} already declared", name));
                }
                Extern{name: self.count, ty: ty.clone() }
            }
            Lam(ref lam)         => self.rename_lam(&mut &**lam, names)?,
            Use{ref name}        => Use{name: name.clone()}
        };
        Ok(())
    }

    fn trans_lam(&mut self, lam: &Lam<VarTy>
                 , module: &mut ir::Module) -> Result<()> {
        self.trans(lam.body(), module)?;
        Ok(())
    }
    
    fn trans(&mut self, expr: &Expr<VarTy>
             , module: &mut ir::Module) -> Result<ir::Expr> {
        use Expr::*;
        let res = match *expr {
            UnitLit    => ir::Expr::UnitLit,
            I32Lit(n)  => ir::Expr::I32Lit(n),
            BoolLit(b) => ir::Expr::BoolLit(b),
            If(ref e)  => {
                let cond  = self.trans(e.cond(), module)?;
                let texpr = self.trans(e.texpr(), module)?;
                let fexpr = self.trans(e.fexpr(), module)?;
                ir::Expr::If{ cond:  Box::new(cond),
                              texpr: Box::new(texpr),
                              fexpr: Box::new(fexpr)
                }
            },
            ref expr   => { println!("NOTHANDLED\n{:?} not handled", expr);
                            unimplemented!() },
        };
        Ok(res)
    }
}
