use std::collections::HashMap;

use crate::monoir;
use crate::{Error, Result};
use cranelift::codegen::{self, ir::{self, Function, Value}};
use cranelift::frontend::Variable;
use cranelift::prelude::{FunctionBuilder, Signature, Block, InstBuilder};
use cranelift_module::{FuncId, Module};

pub(super) struct FunctionTranslator<'a> {
    module: &'a super::module::ModuleTranslator,
    func_ids: &'a HashMap<u32, FuncId>,
    vars: HashMap<u32, Value>,
}

impl<'a> FunctionTranslator<'a> {

    pub(super) fn new(
        module: &'a super::module::ModuleTranslator,
        func_ids: &'a HashMap<u32, FuncId>,
    ) -> Self {
        Self {
            module,
            func_ids,
            vars: HashMap::new(),
        }
    }
    
    pub(super) fn emit_func(
        &mut self,
        bind: &monoir::Bind,
        sig: &Signature,
    ) -> Result<Function> {
        let mut function = cranelift::frontend::FunctionBuilderContext::new();
        let mut func = ir::Function::with_name_signature(
            cranelift::prelude::ExternalName::user(bind.sym.id, 0),
            sig.clone(),
        );
    
        let mut builder = FunctionBuilder::new(&mut func, &mut function);
        let block = self.module.create_entry_block(&mut builder);
        let vars = self.module.setup_params(&mut builder, &bind.sym, block)?;

        self.emit(&bind.expr, &mut builder, block)?;
        Ok(func)
    }

    fn emit(
        &mut self,
        expr: &monoir::Expr,
        builder: &mut FunctionBuilder,
        block: Block,
    ) -> Result<Value> {
        use monoir::Expr::*;
        match expr {
            I32Lit(v) => {
                let ty = self.module.translate_type(&monoir::Type::I32);
                Ok(builder.ins().iconst(ty, *v as i64))
            }
            Let(bind, expr) => {
                let var = self.module.declare_variable(&bind.sym, builder);
                let res = self.emit(&bind.expr, builder, block)?;
                builder.def_var(var, res);
                self.vars.insert(bind.sym.id, res);
                self.emit(expr, builder, block)
            }
            Var(v) => {
                match self.vars.get(&v.id) {
                    Some(v) => Ok(*v),
                    None => Err(Error::new(format!("Variable {v:?} could not be found {:?}", self.vars))),
                }
            }
            App(var, args) => {
                match **var {
                    Var(ref func_sym) => {
                        let func_id = match self.func_ids.get(&func_sym.id) {
                            Some(func_id) => func_id,
                            None => {
                                return Err(Error::new(format!("{func_sym:?} was not found")));
                            }
                        };
                        let local_callee = self
                            .module
                            .inner
                            .declare_func_in_func(*func_id, &mut builder.func);

                        let mut arg_values = Vec::new();
                        for arg in args {
                            arg_values.push(self.emit(arg, builder, block)?)
                        }
                        let call = builder.ins().call(local_callee, &arg_values);
                        Ok(builder.inst_results(call)[0])
                    }
                    _ => return Err(Error::new(format!("Only direct calls supported {expr:?}")))
                }
            }
            Lam(lam) => {
                /*
                self.var_env.begin_scope();
                for (i, param) in lam.params().iter().enumerate() {
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
                 */
                let res = self.emit(&lam.body, builder, block)?;
                builder.ins().return_(&[res]);
                Ok(res)
            }
            _ => Err(Error::new(format!("Can not handle {expr:#?}")))
        }
    }            
        
}
