extern crate llvm_sys;

use ::ir;
use ::Result;
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



struct ModuleCodeGen<'a, 'b> {
    module: LLVMModuleRef,
    ir_module: &'b ir::Module,
    context: &'a LLVMContextRef,
    builder: LLVMBuilderRef,
}

impl <'a,'b> ModuleCodeGen<'a,'b> {
    fn new(ir_module: &'b ir::Module, context: &'a mut LLVMContextRef) -> Self {
        let mod_name = ir_module.name().as_bytes().as_ptr() as * const _;
        let module = unsafe{ core::LLVMModuleCreateWithName(mod_name) };
        let builder = unsafe { core::LLVMCreateBuilderInContext(*context) };
        Self {ir_module, module, context, builder}
    }

    fn run(&mut self ) -> Result<()>{
        for ex in self.ir_module.externs() {
            self.cg_extern(ex)?;
        }
        for lam in self.ir_module.lambdas() {
           self.cg_lambda(lam)?;
        }
        Ok(())
    }

    fn get_type(&mut self, ty: &ir::Type) -> LLVMTypeRef {
        use ir::Type::*;
        use ir::BaseType::*;

        unsafe {
            match *ty {
                BaseType(Unit) => core::LLVMVoidTypeInContext(*self.context),
                BaseType(I32)  => core::LLVMInt32TypeInContext(*self.context),
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
                _ => unimplemented!(),
            }
        }
    }
    
    fn cg_proto(&mut self, proto: &ir::FnProto) -> Result<LLVMValueRef> {
        let function_type = self.get_type(proto.name().ty());
        let name = proto.name().name().as_bytes().as_ptr() as *const _;
        let function = unsafe {
            core::LLVMAddFunction(self.module, name, function_type)
        };
        Ok(function)
    }

    fn cg_extern(&mut self, proto: &ir::FnProto) -> Result<()> {
        println!("EXTERN {:?}", proto);
        let proto = self.cg_proto(proto)?;
        Ok(())
    }
    
    fn cg_lambda(&mut self, lam: &ir::Lambda) -> Result<()> {
        use self::llvm_sys::core::*;
        println!("LAMBDA {:?}", lam);

        let proto = self.cg_proto(lam.proto())?;
        let bb = unsafe {
            let name = b"func_entry\0".as_ptr() as *const _;
            let bb = LLVMAppendBasicBlockInContext(*self.context, proto, name);
            LLVMPositionBuilderAtEnd(self.builder, bb);
            bb
        };
        let body = self.cg_expr(lam.body(), bb)?;
        unsafe {
            LLVMBuildRet(self.builder, body);            
        };
        
        Ok(())
    }
    
    fn cg_expr(&mut self, expr: &ir::Expr, bb: LLVMBasicBlockRef)
               -> Result<LLVMValueRef>
    {
        use self::llvm_sys::core::*;
        use ir::Expr::*;
        use ir::Type::*;
        use ir::BaseType::*;
        let val = unsafe {
            match *expr {
                I32Lit(n) => {
                    LLVMConstInt(self.get_type(&BaseType(I32))
                                 , n as u64
                                 , false as i32)
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
            LLVMDumpModule(self.module);
            LLVMDisposeBuilder(self.builder);
            LLVMDisposeModule(self.module);
        }
    }
}
