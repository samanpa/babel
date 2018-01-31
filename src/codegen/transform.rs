//FIXME: rename this module 
extern crate llvm_sys;

use ::monoir;
use ::Result;
use ::scoped_map::ScopedMap;
use std::ffi::CString;
use self::llvm_sys::prelude::*;
use self::llvm_sys::core::*;
use super::prelude::Prelude;

fn label<T: Into<Vec<u8>>>(str: T) -> CString {
    unsafe{ CString::from_vec_unchecked(str.into().to_vec()) }
}
fn to_cstr(str: &String) -> CString {
    label(str.as_str())
}


pub struct LowerToLlvm<'a> {
    module: LLVMModuleRef,
    context: &'a LLVMContextRef,
    builder: LLVMBuilderRef,
    var_env: ScopedMap<u32, LLVMValueRef>,
}

impl <'a> LowerToLlvm<'a> {
    pub fn new(mod_name: &String, context: &'a mut LLVMContextRef) -> Self {
        let mod_name = to_cstr(mod_name);
        let module  = unsafe{ LLVMModuleCreateWithName(mod_name.as_ptr()) };
        let builder = unsafe{ LLVMCreateBuilderInContext(*context) };
        Self{module, context, builder, var_env: ScopedMap::new()}
    }

    pub fn module(&self) -> LLVMModuleRef {
        self.module
    }

    unsafe fn get_type(&mut self, ty: &monoir::Type, param: bool) -> LLVMTypeRef {
        use monoir::Type::*;

        match *ty {
            Unit => LLVMVoidTypeInContext(*self.context),
            I32  => LLVMInt32TypeInContext(*self.context),
            Bool => LLVMInt1TypeInContext(*self.context),
            Function{ref params_ty, ref return_ty} => {
                let return_ty = self.get_type(return_ty, true);
                let mut params_ty : Vec<LLVMTypeRef> = params_ty.iter()
                    .map( |ref ty| self.get_type(ty, true))
                    .collect();
                let is_var_arg = false;
                let fn_ty = LLVMFunctionType(return_ty,
                                             params_ty.as_mut_ptr(),
                                             params_ty.len() as u32,
                                             is_var_arg as LLVMBool);
                // In LLVM the type of a function passed as a parameter is
                //     Pointer(FunctionType)
                if param {
                    const ADDRESS_SPACE : u32 = 0;
                    LLVMPointerType(fn_ty, ADDRESS_SPACE)
                }
                else {
                    fn_ty
                }
            },
        }
    }
    
    fn gen_proto(&mut self, name: &monoir::TermVar) -> Result<LLVMValueRef> {
        unsafe {
            let fn_ty = self.get_type(name.ty(), false);
            let cname = to_cstr(name.name());
            let func  = LLVMAddFunction(self.module, cname.as_ptr(), fn_ty);
            self.var_env.insert(name.id(), func);
            Ok(func)
        }
    }

    pub fn gen_extern(&mut self, name: &monoir::TermVar) -> Result<()> {
        let func = self.gen_proto(name)?;
        let _ = unsafe {
            Prelude::emit(name, func, self.module, self.builder)?
        };
        Ok(())
    }
    
    pub fn gen_func(&mut self, func: &monoir::Func) -> Result<()> {
        let llfunc = self.gen_proto(func.name())?;
        let bb     = self.add_bb(llfunc, "func_entry");
        self.emit(func.body(), bb, llfunc)?;
        
        Ok(())
    }

    fn add_bb(&mut self, func: LLVMValueRef, name: &str) -> LLVMBasicBlockRef {
        unsafe {
            LLVMAppendBasicBlockInContext(*self.context, func, label(name).as_ptr())
        }
    }
    
    fn emit(&mut self, expr: &monoir::Expr, bb: LLVMBasicBlockRef
            , func: LLVMValueRef) -> Result<LLVMValueRef>
    {
        use monoir::Expr::*;
        use monoir::Type::*;
        let val = unsafe {
            match *expr {
                I32Lit(n) => {
                    let ty = self.get_type(&I32, false);
                    LLVMConstInt(ty, n as u64, false as i32)
                },
                BoolLit(b) => {
                    let ty = self.get_type(&Bool, false);
                    LLVMConstInt(ty, b as u64, false as i32)
                },
                App(ref callee, ref args) => {
                    let callee = self.emit(callee, bb, func)?; 
                    let mut llargs = Vec::new();
                    for arg in args {
                        let llarg = self.emit(arg, bb, func)?;
                        llargs.push(llarg);
                    }
                    let call = LLVMBuildCall(self.builder
                                             , callee
                                             , llargs.as_mut_ptr()
                                             , llargs.len() as u32
                                             , label("call_func").as_ptr());
                    call
                }
                Var(ref var) => {
                    let llvar = self.var_env.get(&var.id());
                    match llvar {
                        None => {
                            let msg = format!("Var not found {:?} {}", var, var.id());
                            return Err(::Error::new(msg));
                        },
                        Some(v) => *v
                    }
                }
                If(ref cond, ref texpr, ref fexpr, ref ty) => {
                    //codegen the condition
                    let cond = self.emit(cond, bb, func)?;

                    //Generate the blocks
                    let then_bb = self.add_bb(func, "then");
                    let else_bb = self.add_bb(func, "else");
                    let phi_bb  = self.add_bb(func, "phi");

                    //build conditional branch
                    LLVMBuildCondBr(self.builder, cond, then_bb, else_bb);

                    //then
                    LLVMPositionBuilderAtEnd(self.builder, then_bb);
                    let texpr = self.emit(texpr, then_bb, func)?;
                    LLVMBuildBr(self.builder, phi_bb);

                    //else
                    LLVMPositionBuilderAtEnd(self.builder, else_bb);
                    let fexpr = self.emit(fexpr, else_bb, func)?;
                    LLVMBuildBr(self.builder, phi_bb);

                    //merge
                    LLVMPositionBuilderAtEnd(self.builder, phi_bb);
                    let mut incoming_vals = vec![texpr, fexpr];
                    let mut incoming_bbs  = vec![then_bb, else_bb];
                    //hack to get type
                    let ty = self.get_type(ty, false);
                    let phi = LLVMBuildPhi(self.builder, ty, label("phi").as_ptr());
                    LLVMAddIncoming(phi,
                                    incoming_vals.as_mut_ptr(),
                                    incoming_bbs.as_mut_ptr(),
                                    incoming_vals.len() as u32);
                                    
                    LLVMPositionBuilderAtEnd(self.builder, phi_bb);

                    phi
                },
                Lam(ref lam) => {
                    self.var_env.begin_scope();
                    for (i,param) in lam.params().iter().enumerate() {
                        let val = LLVMGetParam(func, i as u32);
                        LLVMSetValueName(val, to_cstr(param.name()).as_ptr());
                        self.var_env.insert(param.id(), val);
                    }
                    LLVMPositionBuilderAtEnd(self.builder, bb);
                    let body = self.emit(lam.body(), bb, func)?;
                    LLVMBuildRet(self.builder, body);
                    
                    //LLVMVerifyFunction(self.to_ref(), action) > 0
                    self.var_env.end_scope();
                    body
                }
                Let( ref id, ref expr, ref body) => {
                    //FIXME: do rest
                    let expr = self.emit(expr, bb, func)?;
                    expr
                }
                _ => unimplemented!(),
            }
        };
        Ok(val)
    }
}

impl <'a> Drop for LowerToLlvm<'a> {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeBuilder(self.builder);
            LLVMDisposeModule(self.module);
        }
    }
}
