extern crate llvm_sys;
extern crate libc;

use self::llvm_sys::prelude::*;

use ::Result;
use std::ptr;
use std::ffi::{CStr, CString};

extern "C" fn fatal_error(reason: *const self::libc::c_char) {
    println!("LLVM Fatal error {:?}", reason);
}

pub fn emit(module: LLVMModuleRef, output_file: &String) -> Result<String> {
    let cpu = CString::new("x86-64").expect("invalid cpu");
    let feature = CString::new("").expect("invalid feature");
    let object_file = output_file.clone() + ".o";
    let output_file = CString::new(object_file.as_str())
        .expect("invalid file");

    unsafe {
        use self::llvm_sys::target::*;
        use self::llvm_sys::target_machine::*;
        use self::llvm_sys::initialization::*;
        use self::llvm_sys::error_handling::*;
        use self::llvm_sys::core::*;


        LLVM_InitializeNativeTarget();
        LLVM_InitializeAllTargets();
        LLVM_InitializeAllTargetMCs();
        LLVM_InitializeAllTargetInfos();
        LLVM_InitializeAllAsmPrinters();
        LLVM_InitializeAllAsmParsers();
        LLVMEnablePrettyStackTrace();
        LLVMInstallFatalErrorHandler(Some(fatal_error));

        let passreg = LLVMGetGlobalPassRegistry();
        LLVMInitializeCore(passreg);
        LLVMInitializeTransformUtils(passreg);
        LLVMInitializeScalarOpts(passreg);
        LLVMInitializeObjCARCOpts(passreg);
        LLVMInitializeVectorization(passreg);
        LLVMInitializeInstCombine(passreg);
        LLVMInitializeIPO(passreg);
        LLVMInitializeInstrumentation(passreg);
        LLVMInitializeAnalysis(passreg);
        LLVMInitializeIPA(passreg);
        LLVMInitializeCodeGen(passreg);
        LLVMInitializeTarget(passreg);
        
        let triple = LLVMGetDefaultTargetTriple();
        let mut error_str = ptr::null_mut();
        let mut target = LLVMGetFirstTarget();
        let res = LLVMGetTargetFromTriple(triple, &mut target, &mut error_str);
        if res != 0 {
            let x = CStr::from_ptr(error_str);
            LLVMDisposeMessage(error_str);
            panic!("Could not get target! {:?}", x);
        }
        let opt_level = LLVMCodeGenOptLevel::LLVMCodeGenLevelAggressive;
        let reloc_mode = LLVMRelocMode::LLVMRelocDefault;
        let code_model = LLVMCodeModel::LLVMCodeModelDefault;
        let target_machine = LLVMCreateTargetMachine(target, triple, cpu.as_ptr(), feature.as_ptr(), opt_level, reloc_mode, code_model);
        let file_type = LLVMCodeGenFileType::LLVMObjectFile;

        let res = LLVMTargetMachineEmitToFile(target_machine
                                              , module
                                              , output_file.as_ptr() as *mut i8
                                              , file_type
                                              , &mut error_str);
        if res == 1 {
            let x = CStr::from_ptr(error_str);
            println!("Could not emit file! {:?}", x);
            LLVMDisposeMessage(error_str);
        }
    }
    
    Ok(object_file)
}
