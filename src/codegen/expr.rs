use std::collections::HashMap;

use crate::monoir::{self, Bind, Expr, Type};
use crate::{Error, Result, Vector};
use cranelift::codegen::ir::{self, Function, Value};
use cranelift::prelude::{FunctionBuilder, InstBuilder, Signature};
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

    pub(super) fn emit_func(&mut self, bind: &Bind, sig: &Signature) -> Result<Function> {
        let mut function = cranelift::frontend::FunctionBuilderContext::new();
        let mut func = ir::Function::with_name_signature(
            cranelift::prelude::ExternalName::user(bind.sym.id, 0),
            sig.clone(),
        );

        let mut builder = FunctionBuilder::new(&mut func, &mut function);
        self.emit(&bind.expr, &mut builder)?;
        Ok(func)
    }

    fn emit_indirect(
        &mut self,
        ty: &Type,
        var: &Expr,
        args: &[Expr],
        builder: &mut FunctionBuilder,
    ) -> Result<Value> {
        let callee = self.emit(var, builder)?;
        let args = Vector::map(args, |arg| self.emit(arg, builder))?;
        let sig = self.module.translate_sig(ty)?;
        let sigref = builder.import_signature(sig);
        let call = builder.ins().call_indirect(sigref, callee, &args);
        Ok(builder.inst_results(call)[0])
    }

    fn emit(&mut self, expr: &Expr, builder: &mut FunctionBuilder) -> Result<Value> {
        use monoir::Expr::*;
        match expr {
            I32Lit(v) => {
                let ty = self.module.translate_type(&monoir::Type::I32);
                Ok(builder.ins().iconst(ty, *v as i64))
            }
            Let(bind, expr) => {
                let var = self.module.declare_variable(&bind.sym, builder);
                let res = self.emit(&bind.expr, builder)?;
                builder.def_var(var, res);
                self.vars.insert(bind.sym.id, res);
                self.emit(expr, builder)
            }
            Var(v) => match self.vars.get(&v.id) {
                Some(v) => Ok(*v),
                None => {
                    // Check to see if it is a function
                    match self.func_ids.get(&v.id) {
                        Some(func_id) => {
                            let module = self.module;
                            let func_ref =
                                module.inner.declare_func_in_func(*func_id, builder.func);
                            let ptr_ty = self.module.pointer_ty();
                            let val = builder.ins().func_addr(ptr_ty, func_ref);
                            self.vars.insert(v.id, val);
                            Ok(val)
                        }
                        None => Err(Error::new(format!(
                            "Variable {v:?} could not be found {:?}",
                            self.vars
                        ))),
                    }
                }
            },
            App(ty, var, args) => match **var {
                Var(ref func_sym) => {
                    let func_id = match self.func_ids.get(&func_sym.id) {
                        Some(func_id) => func_id,
                        None => return self.emit_indirect(ty, var, args, builder),
                    };
                    let local_callee = self
                        .module
                        .inner
                        .declare_func_in_func(*func_id, builder.func);

                    let args = Vector::map(args, |arg| self.emit(arg, builder))?;
                    let call = builder.ins().call(local_callee, &args);
                    Ok(builder.inst_results(call)[0])
                }
                _ => self.emit_indirect(ty, var, args, builder),
            },
            If(if_) => {
                let cond = self.emit(&if_.cond, builder)?;
                let ty = self.module.translate_type(&if_.ty);

                let then_block = builder.create_block();
                let else_block = builder.create_block();
                let merge_block = builder.create_block();

                // Add block parameter for the return value
                builder.append_block_param(merge_block, ty);

                // conditional branch to else block
                builder.ins().brz(cond, else_block, &[]);
                // Fall through to then block.
                builder.ins().jump(then_block, &[]);

                builder.switch_to_block(then_block);
                builder.seal_block(then_block);
                let t_return = self.emit(&if_.texpr, builder)?;
                builder.ins().jump(merge_block, &[t_return]);

                builder.switch_to_block(else_block);
                builder.seal_block(else_block);
                let f_return = self.emit(&if_.fexpr, builder)?;
                builder.ins().jump(merge_block, &[f_return]);

                // Switch to the merge block for subsequent statements.
                builder.switch_to_block(merge_block);
                builder.seal_block(merge_block);

                // Read the value of the if-else by reading the merge block
                // parameter.
                let phi = builder.block_params(merge_block)[0];
                Ok(phi)
            }
            Lam(lam) => {
                println!("{lam:#?}");
                let block = self.module.create_entry_block(builder);
                let vars = self.module.setup_params(builder, &lam.params, block)?;
                for (param, var) in lam.params.iter().zip(vars) {
                    let value = builder.use_var(var);
                    self.vars.insert(param.id, value);
                }

                let res = self.emit(&lam.body, builder)?;
                builder.ins().return_(&[res]);
                Ok(res)
            }
            _ => Err(Error::new(format!("Can not handle {expr:#?}"))),
        }
    }
}
