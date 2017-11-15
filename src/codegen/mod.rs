extern crate llvm_sys;

mod transform;
mod llvm_pass;
mod prelude;

use ::ir;
use ::Result;
use self::llvm_sys::target;
use self::llvm_sys::prelude::*;
use self::llvm_sys::core::*;
use self::transform::LowerToLlvm;
use self::llvm_pass::PassRunner;


pub struct Module {
    module: LLVMModuleRef,
}

impl <'a> Drop for Module {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeModule(self.module);
        }
    }
}


pub struct CodeGen {
    context: LLVMContextRef,
}

impl ::Pass for CodeGen {
    type Input  = Vec<ir::Module>; 
    type Output = Vec<()>;

    fn run(mut self, modules: Self::Input) -> Result<Self::Output> {
        for module in modules {
            self.codegen_module(&module)?;
        }
        Ok(vec![])
    }
}

impl CodeGen {
    pub fn new() -> Self {
        unsafe {
            target::LLVM_InitializeNativeTarget();
            CodeGen{context: LLVMContextCreate(),}
        }
    }

    fn codegen_module(&mut self, module: &ir::Module) -> Result<()> {
        let mut codegen = LowerToLlvm::new(module.name(), &mut self.context);
        let mut pass_runner = PassRunner::new();

        for ex in module.externs() {
            codegen.gen_extern(ex)?;
        }
        for lam in module.lambdas() {
           codegen.gen_lambda(lam)?;
        }
        
        let module = codegen.module();
        pass_runner.run(module)?;

        //LLVMPrintModuleToFile (module, const char *Filename, char **ErrorMessage)
        Ok(())
    }
}

