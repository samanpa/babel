extern crate llvm_sys;

use self::llvm_sys::core::*;
use self::llvm_sys::prelude::*;
use self::llvm_sys::transforms::*;
use crate::Result;
use std;

pub struct PassRunner {}

impl PassRunner {
    pub fn new() -> Self {
        PassRunner {}
    }

    pub fn run(&self, module: LLVMModuleRef) -> Result<()> {
        unsafe {
            use self::llvm_sys::transforms::pass_manager_builder::*;

            let pb = LLVMPassManagerBuilderCreate();
            let fpm = LLVMCreateFunctionPassManagerForModule(module);
            let pm = LLVMCreatePassManager();

            //promote allocas to register
            /*
                        scalar::LLVMAddPromoteMemoryToRegisterPass(fpm);
                        scalar::LLVMAddInstructionCombiningPass(fpm);
                        scalar::LLVMAddNewGVNPass(fpm);
                        scalar::LLVMAddReassociatePass(fpm);
                        scalar::LLVMAddCFGSimplificationPass(fpm);
                        scalar::LLVMAddConstantPropagationPass(fpm);
            */

            ipo::LLVMAddFunctionInliningPass(pm);
            ipo::LLVMAddGlobalDCEPass(pm);
            ipo::LLVMAddIPConstantPropagationPass(pm);

            LLVMPassManagerBuilderSetOptLevel(pb, 3);
            LLVMPassManagerBuilderPopulateFunctionPassManager(pb, fpm);
            LLVMPassManagerBuilderPopulateModulePassManager(pb, pm);

            LLVMFinalizeFunctionPassManager(fpm);

            let mut func = LLVMGetFirstFunction(module);
            while func == std::ptr::null_mut() {
                LLVMRunFunctionPassManager(fpm, func);
                func = LLVMGetNextFunction(func);
            }
            LLVMRunPassManager(pm, module);

            LLVMDisposePassManager(fpm);
        }

        Ok(())
    }
}
