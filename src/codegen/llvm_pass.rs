extern crate llvm_sys;

use std;
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
            let fpm = LLVMCreateFunctionPassManagerForModule(module);
            let pm = LLVMCreatePassManager();
            
            //promote allocas to register
            scalar::LLVMAddPromoteMemoryToRegisterPass(fpm);
            scalar::LLVMAddInstructionCombiningPass(fpm);
            scalar::LLVMAddNewGVNPass(fpm);
            scalar::LLVMAddReassociatePass(fpm);
            scalar::LLVMAddCFGSimplificationPass(fpm);
            scalar::LLVMAddConstantPropagationPass(fpm);

            ipo::LLVMAddFunctionInliningPass(pm);
            ipo::LLVMAddGlobalDCEPass(pm);
            ipo::LLVMAddIPConstantPropagationPass(pm);

            LLVMFinalizeFunctionPassManager(fpm);

            let mut func = LLVMGetFirstFunction(module);
            while func == std::ptr::null_mut() {
                LLVMRunFunctionPassManager(fpm, func);
                func = LLVMGetNextFunction(func);
            }
            LLVMRunPassManager(pm, module);

            LLVMDumpModule(module);

            LLVMDisposePassManager(fpm);
        }

        Ok(())
    }
}
