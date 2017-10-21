// Translates to syntax tree to intermediate representation
//    Called it elaboration in previous incarnation

use ::ast;
use ::ir;
use ::Result;

pub struct Translate {
}

impl ::Pass for Translate {
    type Input = Vec<ast::TopLevel>; //A list of parsed files
    type Output = Vec<ir::Module>;   //A list of modules

    fn run(&mut self, toplevel_vec: Self::Input) -> Result<Self::Output> {
        let mut mods: Vec<ir::Module> = vec![];
        for toplevel in &toplevel_vec {
            self.trans_toplevel(toplevel, &mut mods)?;
        }
        Ok(mods)
    }
}

impl Translate {
    fn new() -> Self {
        Translate{}
    }

    fn trans_toplevel(&mut self
                      , toplevel: &ast::TopLevel
                      , modules: &mut Vec<ir::Module>) -> Result<()> {
        //FIXME: find better way
        let mut module = ir::Module::new("main".to_string());
        for decl in toplevel.decls() {
            self.trans_topdecl(decl, &mut module)?
        }
        modules.push(module);
        Ok(())
    }
    
    fn trans_topdecl(&mut self
                     , decl: &ast::TopDecl
                     , module: &mut ir::Module) -> Result<()> {
        use ast::TopDecl::*;
        let res = match *decl {
            Extern{..}   => (),
            Use{..}      => (),
            Lam(ref lam) => self.trans_lam(&mut &**lam, module)?,
        };
        Ok(())
    }

    fn trans_lam(&mut self, lam: &ast::Lam
                 , module: &mut ir::Module) -> Result<()> {
        println!("{:?}", *lam);
        self.trans(lam.body(), module)?;
        Ok(())
    }
    
    fn trans(&mut self, expr: &ast::Expr
             , module: &mut ir::Module) -> Result<ir::Expr> {
        use ast::Expr::*;
        let res = match *expr {
            UnitLit    => ir::Expr::UnitLit,
            I32Lit(n)  => ir::Expr::I32Lit(n),
            BoolLit(b) => ir::Expr::BoolLit(b),
            If(ref e)  => {
                let cond       = self.trans(e.cond(), module)?;
                let true_expr  = self.trans(e.true_expr(), module)?;
                let false_expr = self.trans(e.false_expr(), module)?;
                ir::Expr::If{ cond: Box::new(cond),
                              true_expr: Box::new(true_expr),
                              false_expr: Box::new(false_expr)
                }
            },
            ref expr   => { println!("NOTHANDLED\n{:?} not handled", expr);
                            unimplemented!() },
        };
        Ok(res)
    }
}
