//FIXME: rename this module

use crate::monoir;
use crate::Result;
use cranelift::codegen::ir::{self, Function, InstBuilder};
use cranelift::frontend::{FunctionBuilder, Variable};
use cranelift::prelude::{Block, Signature};

pub struct Prelude {}

impl Prelude {
    fn setup_params(
        module: &super::translate::ModuleTranslator,
        builder: &mut FunctionBuilder<'_>,
        symbol: &monoir::Symbol,
        block: Block,
    ) -> Result<Vec<Variable>> {
        let symbols = match &symbol.ty {
            monoir::Type::Function { params_ty, .. } => params_ty
                .iter()
                .enumerate()
                .map(|(id, ty)| monoir::Symbol {
                    name: std::rc::Rc::new(format!("param{id}")),
                    ty: ty.clone(),
                    id: id as u32,
                })
                .collect::<Vec<_>>(),
            _ => todo!(),
        };
        Ok(module.setup_params(&symbols, builder, block))
    }

    pub(super) fn emit(
        &self,
        trans: &super::translate::Translator,
        sym: &monoir::Symbol,
        sig: &Signature,
    ) -> Result<Option<Function>> {
        use cranelift::codegen::ir::condcodes::IntCC;

        let mut function = cranelift::frontend::FunctionBuilderContext::new();
        let mut func = ir::Function::with_name_signature(
            cranelift::prelude::ExternalName::user(sym.id, 0),
            sig.clone(),
        );

        let mut builder = FunctionBuilder::new(&mut func, &mut function);
        let res = match sym.name().as_str() {
            "i32_add" => {
                let block = trans.module.create_entry_block(&mut builder);
                let vars = Self::setup_params(&trans.module, &mut builder, sym, block)?;
                let p0 = builder.use_var(vars[0]);
                let p1 = builder.use_var(vars[1]);
                let res = builder.ins().iadd(p0, p1);
                builder.ins().return_(&[res]);
                Some(func)
            }
            "i32_sub" => {
                let block = trans.module.create_entry_block(&mut builder);
                let vars = Self::setup_params(&trans.module, &mut builder, sym, block)?;
                let p0 = builder.use_var(vars[0]);
                let p1 = builder.use_var(vars[1]);
                let res = builder.ins().isub(p0, p1);
                builder.ins().return_(&[res]);
                Some(func)
            }
            "i32_mul" => {
                let block = trans.module.create_entry_block(&mut builder);
                let vars = Self::setup_params(&trans.module, &mut builder, sym, block)?;
                let p0 = builder.use_var(vars[0]);
                let p1 = builder.use_var(vars[1]);
                let res = builder.ins().imul(p0, p1);
                builder.ins().return_(&[res]);
                Some(func)
            }
            "i32_div" => {
                let block = trans.module.create_entry_block(&mut builder);
                let vars = Self::setup_params(&trans.module, &mut builder, sym, block)?;
                let p0 = builder.use_var(vars[0]);
                let p1 = builder.use_var(vars[1]);
                let res = builder.ins().sdiv(p0, p1);
                builder.ins().return_(&[res]);
                Some(func)
            }
            "i32_mod" => {
                let block = trans.module.create_entry_block(&mut builder);
                let vars = Self::setup_params(&trans.module, &mut builder, sym, block)?;
                let p0 = builder.use_var(vars[0]);
                let p1 = builder.use_var(vars[1]);
                let res = builder.ins().srem(p0, p1);
                builder.ins().return_(&[res]);
                Some(func)
            }
            "i32_lt" => {
                let block = trans.module.create_entry_block(&mut builder);
                let vars = Self::setup_params(&trans.module, &mut builder, sym, block)?;
                let p0 = builder.use_var(vars[0]);
                let p1 = builder.use_var(vars[1]);
                let res = builder.ins().icmp(IntCC::SignedLessThan, p0, p1);
                builder.ins().return_(&[res]);
                Some(func)
            }
            "i32_gt" => {
                let block = trans.module.create_entry_block(&mut builder);
                let vars = Self::setup_params(&trans.module, &mut builder, sym, block)?;
                let p0 = builder.use_var(vars[0]);
                let p1 = builder.use_var(vars[1]);
                let res = builder.ins().icmp(IntCC::SignedGreaterThan, p0, p1);
                builder.ins().return_(&[res]);
                Some(func)
            }
            "i32_eq" => {
                let block = trans.module.create_entry_block(&mut builder);
                let vars = Self::setup_params(&trans.module, &mut builder, sym, block)?;
                let p0 = builder.use_var(vars[0]);
                let p1 = builder.use_var(vars[1]);
                let res = builder.ins().icmp(IntCC::Equal, p0, p1);
                builder.ins().return_(&[res]);
                Some(func)
            }
            _ => None,
        };
        Ok(res)
    }
}
