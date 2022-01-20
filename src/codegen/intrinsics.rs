use crate::monoir;
use crate::{Error, Result};
use cranelift::codegen::ir::{self, Function, InstBuilder};
use cranelift::frontend::FunctionBuilder;
use cranelift::prelude::Signature;

fn params(func: &monoir::Symbol) -> Result<Vec<monoir::Symbol>> {
    match &func.ty {
        monoir::Type::Function { params_ty, .. } => {
            let params = params_ty
                .iter()
                .enumerate()
                .map(|(id, ty)| monoir::Symbol {
                    name: std::rc::Rc::new(format!("param {id}")),
                    ty: ty.clone(),
                    id: id as u32,
                })
                .collect::<Vec<_>>();
            Ok(params)
        }
        _ => Err(Error::new(format!("{func:?} is not a function. "))),
    }
}

pub(super) fn emit(
    module: &super::module::ModuleTranslator,
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
    match sym.name.as_str() {
        "i32_add" => {
            let block = module.create_entry_block(&mut builder);
            let vars = module.setup_params(&mut builder, &params(sym)?, block)?;
            let p0 = builder.use_var(vars[0]);
            let p1 = builder.use_var(vars[1]);
            let res = builder.ins().iadd(p0, p1);
            builder.ins().return_(&[res]);
        }
        "i32_sub" => {
            let block = module.create_entry_block(&mut builder);
            let vars = module.setup_params(&mut builder, &params(sym)?, block)?;
            let p0 = builder.use_var(vars[0]);
            let p1 = builder.use_var(vars[1]);
            let res = builder.ins().isub(p0, p1);
            builder.ins().return_(&[res]);
        }
        "i32_mul" => {
            let block = module.create_entry_block(&mut builder);
            let vars = module.setup_params(&mut builder, &params(sym)?, block)?;
            let p0 = builder.use_var(vars[0]);
            let p1 = builder.use_var(vars[1]);
            let res = builder.ins().imul(p0, p1);
            builder.ins().return_(&[res]);
        }
        "i32_div" => {
            let block = module.create_entry_block(&mut builder);
            let vars = module.setup_params(&mut builder, &params(sym)?, block)?;
            let p0 = builder.use_var(vars[0]);
            let p1 = builder.use_var(vars[1]);
            let res = builder.ins().sdiv(p0, p1);
            builder.ins().return_(&[res]);
        }
        "i32_mod" => {
            let block = module.create_entry_block(&mut builder);
            let vars = module.setup_params(&mut builder, &params(sym)?, block)?;
            let p0 = builder.use_var(vars[0]);
            let p1 = builder.use_var(vars[1]);
            let res = builder.ins().srem(p0, p1);
            builder.ins().return_(&[res]);
        }
        "i32_lt" => {
            let block = module.create_entry_block(&mut builder);
            let vars = module.setup_params(&mut builder, &params(sym)?, block)?;
            let p0 = builder.use_var(vars[0]);
            let p1 = builder.use_var(vars[1]);
            let res = builder.ins().icmp(IntCC::SignedLessThan, p0, p1);
            builder.ins().return_(&[res]);
        }
        "i32_gt" => {
            let block = module.create_entry_block(&mut builder);
            let vars = module.setup_params(&mut builder, &params(sym)?, block)?;
            let p0 = builder.use_var(vars[0]);
            let p1 = builder.use_var(vars[1]);
            let res = builder.ins().icmp(IntCC::SignedGreaterThan, p0, p1);
            builder.ins().return_(&[res]);
        }
        "i32_eq" => {
            let block = module.create_entry_block(&mut builder);
            let vars = module.setup_params(&mut builder, &params(sym)?, block)?;
            let p0 = builder.use_var(vars[0]);
            let p1 = builder.use_var(vars[1]);
            let res = builder.ins().icmp(IntCC::Equal, p0, p1);
            builder.ins().return_(&[res]);
        }
        _ => return Ok(None),
    };
    Ok(Some(func))
}
