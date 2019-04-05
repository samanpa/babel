extern crate llvm_sys;

mod emit;
mod llvm_pass;
mod prelude;
mod transform;

use self::llvm_pass::PassRunner;
use self::llvm_sys::core::*;
use self::llvm_sys::prelude::*;
use self::llvm_sys::target;
use self::transform::LowerToLlvm;
use crate::monoir;
use crate::{Result, Vector};

pub struct Module {
    module: LLVMModuleRef,
}

impl<'a> Drop for Module {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeModule(self.module);
        }
    }
}

pub struct CodeGen {
    context: LLVMContextRef,
    output_file: String,
}

impl crate::Pass for CodeGen {
    type Input = Vec<monoir::Module>;
    type Output = Vec<String>;

    fn run(mut self, modules: Self::Input) -> Result<Self::Output> {
        Vector::map(&modules, |module| self.codegen_module(&module))
    }
}

impl CodeGen {
    pub fn new(output_file: String) -> Self {
        unsafe {
            target::LLVM_InitializeNativeTarget();
            CodeGen {
                context: LLVMContextCreate(),
                output_file,
            }
        }
    }

    fn codegen_module(&mut self, module: &monoir::Module) -> Result<String> {
        let mut codegen = LowerToLlvm::new(module.name(), &mut self.context);
        let pass_runner = PassRunner::new();

        for ex in module.externs() {
            codegen.gen_extern(ex)?;
        }
        for func in module.funcs() {
            codegen.gen_func(func)?;
        }

        let module = codegen.module();
        pass_runner.run(module)?;

        //unsafe{ LLVMDumpModule(module)};
        let object_file = emit::emit(module, &self.output_file)?;
        Ok(object_file)
    }
}
