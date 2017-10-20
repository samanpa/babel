// Translates to syntax tree to intermediate representation
//    Called it elaboration in previous incarnation
//    Named after the corresponding from Haskell pass

use ::ast;
use ::ir;
use ::Result;

pub struct Desugar {}
struct Context {
    modules: Vec<ir::Module>,
}

//Is this optimal?
trait Translate {
    type Context;
    type Output;
    fn translate(&mut self, env: &mut Self::Context) -> Result<Self::Output>;
}

impl ::Pass for Desugar {
    type Input = Vec<ast::TopLevel>; //A list of parsed files
    type Output = Vec<ir::Module>;   //A list of modules

    fn run(&mut self, toplevel_vec: Self::Input) -> Result<Self::Output> {
        let mut env = Context{ modules: vec![] };
        for mut toplevel in toplevel_vec {
            let _ = Translate::translate(&mut toplevel, &mut env)?;
        }   
        Ok(env.modules)
    }
}

impl Translate for ast::TopLevel {
    type Context = Context;
    type Output = ();

    fn translate(&mut self, env: &mut Self::Context) -> Result<Self::Output> {
        //FIXME: find better way
        let mut module = ir::Module::new("main".to_string());
        for mut decl in self.decls() {
            Translate::translate(&mut decl, &mut module)?
        }
        env.modules.push(module);
        Ok(())
    }
}

impl <'a> Translate for &'a ast::TopDecl {
    type Context = ir::Module;
    type Output = ();

    fn translate(&mut self, module: &mut ir::Module) -> Result<Self::Output> {
        use ast::TopDecl::*;
        use self::Translate;
        let res = match *self {
            &Extern{..}   => (),
            &Use{..}      => (),
            &Lam(ref lam) => (&mut &**lam).translate(module)?,
        };
        Ok(())
    }
}

impl <'a> Translate for &'a ast::Lam {
    type Context = ir::Module;
    //type Output = ir::Lambda;
    type Output = ();

    fn translate(&mut self, module: &mut ir::Module) -> Result<Self::Output> {
        println!("{:?}", *self);
        let expr = (&mut self.body()).translate(module)?;
        
        Ok(())
    }
}

impl <'a> Translate for &'a ast::Expr {
    type Context = ir::Module;
    type Output = ir::Expr;

    fn translate(&mut self, module: &mut ir::Module) -> Result<Self::Output> {
        use ast::Expr::*;
        let res = match *self {
            &UnitLit    => ir::Expr::UnitLit,
            &I32Lit(n)  => ir::Expr::I32Lit(n),
            &BoolLit(b) => ir::Expr::BoolLit(b),
            &If(ref e) => {
                let cond       = e.cond().translate(module)?;
                let true_expr  = e.true_expr().translate(module)?;
                let false_expr = e.false_expr().translate(module)?;
                ir::Expr::If{ cond: Box::new(cond),
                              true_expr: Box::new(true_expr),
                              false_expr: Box::new(false_expr)
                }
            },
            expr        => { println!("NOTHANDLED\n{:?} not handled", expr);
                             unimplemented!() },
        };
        Ok(res)
    }
}
