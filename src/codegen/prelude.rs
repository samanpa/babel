//FIXME: rename this module 
extern crate llvm_sys;

use ::ir;
use ::Result;
use std::ffi::CString;
use self::llvm_sys::prelude::*;
use self::llvm_sys::core::*;
use self::llvm_sys::*;

pub struct Prelude{}

fn label<T: Into<Vec<u8>>>(str: T) -> CString {
    unsafe{ CString::from_vec_unchecked(str.into().to_vec()) }
}

impl Prelude {
    unsafe fn prepare(context: LLVMContextRef, func: LLVMValueRef
                      , builder: LLVMBuilderRef) {
        let nm = label("intrinsic_entry").as_ptr();
        let bb = LLVMAppendBasicBlockInContext(context, func, nm);
        LLVMPositionBuilderAtEnd(builder, bb);
    }

    pub  unsafe fn emit(proto: &ir::FnProto, func: LLVMValueRef
                       , module: LLVMModuleRef
                       , builder: LLVMBuilderRef) 
                       -> Result<Option<LLVMValueRef>>
    {
        let res = match proto.name().name().as_str() {
            "i32_add" => {
                Self::prepare(LLVMGetModuleContext(module), func, builder);
                let p0  = LLVMGetParam(func, 0);
                let p1  = LLVMGetParam(func, 1);
                let add = LLVMBuildAdd(builder, p0, p1, label("add").as_ptr());
                let res = LLVMBuildRet(builder, add);
                Some(res)
            }
            "i32_sub" => {
                Self::prepare(LLVMGetModuleContext(module), func, builder);
                let p0  = LLVMGetParam(func, 0);
                let p1  = LLVMGetParam(func, 1);
                let add = LLVMBuildSub(builder, p0, p1, label("sub").as_ptr());
                let res = LLVMBuildRet(builder, add);
                Some(res)
            }
            "i32_lt" => {
                Self::prepare(LLVMGetModuleContext(module), func, builder);
                let p0  = LLVMGetParam(func, 0);
                let p1  = LLVMGetParam(func, 1);
                let op  = LLVMIntPredicate::LLVMIntSGT;
                let add = LLVMBuildICmp(builder, op, p0, p1, label("lt").as_ptr());
                let res = LLVMBuildRet(builder, add);
                Some(res)
            }
            _ => None
        };
        Ok(res)
    }
}
