use ::ir;
use ::Result;

pub struct CodeGen {
}

impl ::Pass for CodeGen {
    type Input  = Vec<ir::Module>; 
    type Output = Vec<()>;

    fn run(&mut self, modules: Self::Input) -> Result<Self::Output> {
        for module in modules {
            self.codegen_module(&module)?;
        }
        Ok(vec![])
    }
}

impl CodeGen {
    pub fn new() -> Self {
        CodeGen{}
    }

    fn codegen_module(&mut self, module: &ir::Module) -> Result<()> {
        for ex in module.externs() {
            self.codegen_extern(ex)?;
        }
        for lam in module.lambdas() {
            self.codegen_lambda(lam)?;
        }
        Ok(())
    }
    
    fn codegen_extern(&mut self, proto: &ir::FnProto) -> Result<()> {
        println!("{:?}", proto);
        Ok(())
    }

    fn codegen_lambda(&mut self, lam: &ir::Lambda) -> Result<()> {
        println!("{:?}", lam);
        Ok(())
    }

}
