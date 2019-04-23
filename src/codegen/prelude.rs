//FIXME: rename this module
extern crate llvm_sys;

use self::llvm_sys::core::*;
use self::llvm_sys::prelude::*;
use self::llvm_sys::*;
use crate::monoir;
use crate::Result;
use std::ffi::CString;

pub struct Prelude {}

fn label<T: Into<Vec<u8>>>(str: T) -> CString {
    unsafe { CString::from_vec_unchecked(str.into().to_vec()) }
}

impl Prelude {
    unsafe fn prepare(context: LLVMContextRef, func: LLVMValueRef, builder: LLVMBuilderRef) {
        let nm = label("intrinsic_entry").as_ptr();
        let bb = LLVMAppendBasicBlockInContext(context, func, nm);
        LLVMPositionBuilderAtEnd(builder, bb);
    }

    pub unsafe fn emit(
        sym: &monoir::Symbol,
        func: LLVMValueRef,
        module: LLVMModuleRef,
        builder: LLVMBuilderRef,
    ) -> Result<Option<LLVMValueRef>> {
        let res = match sym.name().as_str() {
            "i32_add" => {
                Self::prepare(LLVMGetModuleContext(module), func, builder);
                let p0 = LLVMGetParam(func, 0);
                let p1 = LLVMGetParam(func, 1);
                let add = LLVMBuildAdd(builder, p0, p1, label("add").as_ptr());
                let res = LLVMBuildRet(builder, add);
                Some(res)
            }
            "i32_sub" => {
                Self::prepare(LLVMGetModuleContext(module), func, builder);
                let p0 = LLVMGetParam(func, 0);
                let p1 = LLVMGetParam(func, 1);
                let add = LLVMBuildSub(builder, p0, p1, label("sub").as_ptr());
                let res = LLVMBuildRet(builder, add);
                Some(res)
            }
            "i32_mul" => {
                Self::prepare(LLVMGetModuleContext(module), func, builder);
                let p0 = LLVMGetParam(func, 0);
                let p1 = LLVMGetParam(func, 1);
                let mul = LLVMBuildMul(builder, p0, p1, label("mul").as_ptr());
                let res = LLVMBuildRet(builder, mul);
                Some(res)
            }
            "i32_div" => {
                Self::prepare(LLVMGetModuleContext(module), func, builder);
                let p0 = LLVMGetParam(func, 0);
                let p1 = LLVMGetParam(func, 1);
                let div = LLVMBuildSDiv(builder, p0, p1, label("div").as_ptr());
                let res = LLVMBuildRet(builder, div);
                Some(res)
            }
            "i32_mod" => {
                Self::prepare(LLVMGetModuleContext(module), func, builder);
                let p0 = LLVMGetParam(func, 0);
                let p1 = LLVMGetParam(func, 1);
                let rem = LLVMBuildSRem(builder, p0, p1, label("rem").as_ptr());
                let res = LLVMBuildRet(builder, rem);
                Some(res)
            }
            "i32_lt" => {
                Self::prepare(LLVMGetModuleContext(module), func, builder);
                let p0 = LLVMGetParam(func, 0);
                let p1 = LLVMGetParam(func, 1);
                let op = LLVMIntPredicate::LLVMIntSLT;
                let add = LLVMBuildICmp(builder, op, p0, p1, label("lt").as_ptr());
                let res = LLVMBuildRet(builder, add);
                Some(res)
            }
            "i32_gt" => {
                Self::prepare(LLVMGetModuleContext(module), func, builder);
                let p0 = LLVMGetParam(func, 0);
                let p1 = LLVMGetParam(func, 1);
                let op = LLVMIntPredicate::LLVMIntSGT;
                let add = LLVMBuildICmp(builder, op, p0, p1, label("gt").as_ptr());
                let res = LLVMBuildRet(builder, add);
                Some(res)
            }
            "i32_eq" => {
                Self::prepare(LLVMGetModuleContext(module), func, builder);
                let p0 = LLVMGetParam(func, 0);
                let p1 = LLVMGetParam(func, 1);
                let op = LLVMIntPredicate::LLVMIntEQ;
                let add = LLVMBuildICmp(builder, op, p0, p1, label("eq").as_ptr());
                let res = LLVMBuildRet(builder, add);
                Some(res)
            }
            _ => None,
        };
        Ok(res)
    }
}
