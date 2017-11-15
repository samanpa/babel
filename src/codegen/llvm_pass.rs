extern crate llvm_sys;

use ::Result;
use self::llvm_sys::prelude::*;
use self::llvm_sys::core::*;
use self::llvm_sys::transforms::*;

pub struct PassRunner {
}

impl PassRunner {
    pub fn new() -> Self {
        PassRunner {}
    }

    pub fn run(&self, module: LLVMModuleRef ) -> Result<()>{
        unsafe {
            let pm = LLVMCreateFunctionPassManagerForModule(module);
            
            //promote allocas to register
            scalar::LLVMAddPromoteMemoryToRegisterPass(pm);

            ipo::LLVMAddGlobalDCEPass(pm);
            
            scalar::LLVMAddInstructionCombiningPass(pm);

            //ipo::LLVMAddFunctionInliningPass(pm);

            LLVMFinalizeFunctionPassManager(pm);

            LLVMDumpModule(module);

            LLVMDisposePassManager(pm);
        }

        Ok(())
    }
}
