extern crate llvm_sys;

use ::ir;
use ::Result;
use ::scoped_map::ScopedMap;
use self::llvm_sys::*;
use self::llvm_sys::prelude::*;

pub struct CodeGen {
    context: LLVMContextRef,
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
        unsafe {
            target::LLVM_InitializeNativeTarget();
            CodeGen{context: core::LLVMContextCreate(),}
        }
    }

    fn codegen_module(&mut self, module: &ir::Module) -> Result<()> {
        let mut codegen = ModuleCodeGen::new(module, &mut self.context);
        codegen.run()
    }
}

fn label(str: &str) -> * const i8 {
    str.as_ptr() as *const _
}


struct ModuleCodeGen<'a, 'b> {
    module: LLVMModuleRef,
    ir_module: &'b ir::Module,
    context: &'a LLVMContextRef,
    builder: LLVMBuilderRef,
    var_env: ScopedMap<u32, LLVMValueRef>,
}

impl <'a,'b> ModuleCodeGen<'a,'b> {
    fn new(ir_module: &'b ir::Module, context: &'a mut LLVMContextRef) -> Self {
        //FIXME: Add NUL byte
        let mod_name = ir_module.name().as_bytes().as_ptr() as * const _;
        let module   = unsafe{ core::LLVMModuleCreateWithName(mod_name) };
        let builder  = unsafe{ core::LLVMCreateBuilderInContext(*context) };
        Self{ir_module, module, context, builder, var_env: ScopedMap::new()}
    }

    fn run(&mut self ) -> Result<()>{
        for ex in self.ir_module.externs() {
            self.cg_extern(ex)?;
        }
        for lam in self.ir_module.lambdas() {
           self.cg_lambda(lam)?;
        }
        unsafe {
            self::llvm_sys::core::LLVMDumpModule(self.module);
        }
        Ok(())
    }

    unsafe fn get_type(&mut self, ty: &ir::Type) -> LLVMTypeRef {
        use ir::Type::*;
        use ir::BaseType::*;

        match *ty {
            BaseType(Unit) => core::LLVMVoidTypeInContext(*self.context),
            BaseType(I32)  => core::LLVMInt32TypeInContext(*self.context),
            BaseType(Bool) => core::LLVMInt1TypeInContext(*self.context),
            FunctionType{ref params_ty, ref return_ty} => {
                let return_ty = self.get_type(return_ty);
                let mut params_ty : Vec<LLVMTypeRef> = params_ty.iter()
                    .map( |ref ty| self.get_type(ty))
                    .collect();
                let is_var_arg = false;
                core::LLVMFunctionType(return_ty,
                                       params_ty.as_mut_ptr(),
                                       params_ty.len() as u32,
                                       is_var_arg as LLVMBool)
            },
            PointerType(ref ty) => {
                let ty = self.get_type(ty);
                const ADDRESS_SPACE : u32 = 0;
                core::LLVMPointerType(ty, ADDRESS_SPACE)
            }
        }
    }
    
    fn cg_proto(&mut self, proto: &ir::FnProto) -> Result<LLVMValueRef> {
        unsafe {
            let function_type = self.get_type(proto.name().ty());
            let name = proto.name().name().as_bytes().as_ptr() as * const _;
            let func = core::LLVMAddFunction(self.module, name, function_type);
            for (i,param) in proto.params().iter().enumerate() {
                let value = core::LLVMGetParam(func, i as u32);
                core::LLVMSetValueName(value, label(param.name().as_str()));
            }
            self.var_env.insert(proto.name().id(), func);
            Ok(func)
        }
    }

    fn cg_extern(&mut self, proto: &ir::FnProto) -> Result<()> {
        let _ = self.cg_proto(proto)?;
        Ok(())
    }
    
    fn cg_lambda(&mut self, lam: &ir::Lambda) -> Result<()> {
        use self::llvm_sys::core::*;

        let params = lam.proto().params();
        let proto  = self.cg_proto(lam.proto())?;
        self.var_env.begin_scope();
        unsafe {
            let nm = label("func_entry");
            let bb = LLVMAppendBasicBlockInContext(*self.context, proto, nm);
            for (i,param) in params.iter().enumerate() {
                let value = core::LLVMGetParam(proto, i as u32);
                self.var_env.insert(param.id(), value);
            }
            LLVMPositionBuilderAtEnd(self.builder, bb);
            let body = self.emit(lam.body(), bb, proto)?;
            LLVMBuildRet(self.builder, body);

            //LLVMVerifyFunction(self.to_ref(), action) > 0
        };
        self.var_env.end_scope();

        Ok(())
    }

    fn add_bb(&mut self, func: LLVMValueRef, bblabel: &str) -> LLVMBasicBlockRef {
        use self::llvm_sys::core::*;
        unsafe {
            LLVMAppendBasicBlockInContext(*self.context, func, label(bblabel))
        }
    }
    
    fn emit(&mut self, expr: &ir::Expr, bb: LLVMBasicBlockRef
            , func: LLVMValueRef) -> Result<LLVMValueRef>
    {
        use self::llvm_sys::core::*;
        use ir::Expr::*;
        use ir::Type::*;
        use ir::BaseType::*;
        let val = unsafe {
            match *expr {
                I32Lit(n) => {
                    let ty = self.get_type(&BaseType(I32));
                    LLVMConstInt(ty, n as u64, false as i32)
                },
                BoolLit(b) => {
                    let ty = self.get_type(&BaseType(Bool));
                    LLVMConstInt(ty, b as u64, false as i32)
                },
                App{ref callee, ref args} => {
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
                                             , label("call_func"));
                    call
                }
                Var( ref var ) => {
                    let llvar = self.var_env.get(&var.id());
                    match llvar {
                        None => {
                            let msg = format!("Var not found {:?} {}", var, var.id());
                            return Err(::Error::new(msg));
                        },
                        Some(v) => *v
                    }
                }
                If{ref cond, ref texpr, ref fexpr} => {
                    //codegen the condition
                    let cond = self.emit(cond, bb, func)?;

                    //Generate the blocks
                    let then_bb = self.add_bb(func, "then\0");
                    let else_bb = self.add_bb(func, "else\0");
                    let phi_bb  = self.add_bb(func, "phi\0");

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
                    let ty = self.get_type(&BaseType(I32));
                    let phi = LLVMBuildPhi(self.builder, ty, label("phi\0"));
                    LLVMAddIncoming(phi,
                                    incoming_vals.as_mut_ptr(),
                                    incoming_bbs.as_mut_ptr(),
                                    incoming_vals.len() as u32);
                                    
                    LLVMPositionBuilderAtEnd(self.builder, phi_bb);

                    phi
                },
                
                _ => unimplemented!(),
            }
        };
        Ok(val)
    }
}
impl <'a,'b> Drop for ModuleCodeGen<'a,'b> {
    fn drop(&mut self) {
        use self::llvm_sys::core::*;
        unsafe {
            LLVMDisposeBuilder(self.builder);
            LLVMDisposeModule(self.module);
        }
    }
}
