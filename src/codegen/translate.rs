use crate::monoir;
use crate::scoped_map::ScopedMap;
use crate::{Error, Result};
use cranelift::codegen;
use cranelift::frontend::Variable;
use cranelift::prelude::FunctionBuilder;
use cranelift_module::{FuncId, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};

pub(super) struct ModuleTranslator {
    inner: ObjectModule,
}

pub(super) struct Translator {
    pub(super) module: ModuleTranslator,
    functions: ScopedMap<u32, FuncId>,
}

impl Translator {
    pub(super) fn new(name: &str) -> Result<Self> {
        use codegen::settings::{self, Configurable};

        let triple = target_lexicon::Triple::host();
        let mut builder = settings::builder();
        builder
            .set("opt_level", "speed_and_size")
            .map_err(|_| Error::new("Could not set opt_level"))?;
        let flags = settings::Flags::new(builder);
        let target_isa = codegen::isa::lookup(triple.clone())
            .map_err(|_| Error::new(format!("Unsupported triple {triple:?}")))?
            .finish(flags);
        let builder = ObjectBuilder::new(
            target_isa,
            format!("{}.o", name),
            cranelift_module::default_libcall_names(),
        )
        .map_err(|_| Error::new("Cannot create cranelift module"))?;
        let inner = ObjectModule::new(builder);
        let module = ModuleTranslator { inner };

        Ok(Self {
            module,
            functions: ScopedMap::new(),
        })
    }

    pub(super) fn translate(mut self, module: monoir::Module) -> Result<ObjectModule> {
        use cranelift_module::Linkage;
        let prelude = super::prelude::Prelude {};
        for extern_func in &module.ext_funcs {
            let sig = self.module.translate_sig(extern_func)?;
            let func = prelude.emit(&self, &extern_func, &sig)?;
            let linkage = if func.is_some() {
                Linkage::Local
            } else {
                Linkage::Import
            };
            let func_id = self.module.declare_func(extern_func, linkage, sig)?;
            self.functions.insert(extern_func.id, func_id);
            if let Some(func) = func {
                self.module.define_function(func_id, func)?;
            }
        }

        /*
        let mut func_ids = Vec::new();
        for bind in module.funcs.as_slice() {
            //println!("{bind:#?}");
            let symbol = &bind.sym;
            let sig = self.module.translate_sig(symbol)?;
            let func_id = self.module.declare_func(&symbol, false, sig)?;
            self.functions.insert(symbol.id, func_id);
            func_ids.push(func_id);
        }
             */

        Ok(self.module.inner)
    }
}

impl ModuleTranslator {
    /// Declare a single variable declaration.
    fn declare_variable(&self, symbol: &monoir::Symbol, builder: &mut FunctionBuilder) -> Variable {
        if symbol.ty == monoir::Type::Unit {
            panic!("{:?} is a unit type", symbol);
        }
        let ty = self.translate_type(&symbol.ty);
        let var = Variable::with_u32(symbol.id);
        builder.declare_var(var, ty);
        var
    }

    fn define_function(&mut self, funcid: FuncId, function: codegen::ir::Function) -> Result<()> {
        println!("{}", function);
        use codegen::{
            binemit::{NullStackMapSink, NullTrapSink},
            settings, Context,
        };
        let flags = settings::Flags::new(settings::builder());
        codegen::verify_function(&function, &flags).unwrap();

        let mut context = Context::for_function(function);

        self.inner
            .define_function(
                funcid,
                &mut context,
                &mut NullTrapSink {},
                &mut NullStackMapSink {},
            )
            .map_err(|e| format!("Could not define function {e:?}"))?;

        Ok(())
    }

    fn pointer_ty(&self) -> codegen::ir::Type {
        self.inner.target_config().pointer_type()
    }

    fn translate_type(&self, ty: &monoir::Type) -> codegen::ir::Type {
        use codegen::ir::types;
        match ty {
            monoir::Type::Unit => todo!(),
            monoir::Type::Bool => types::B1,
            monoir::Type::I32 => types::I32,
            monoir::Type::Function { .. } => self.pointer_ty(),
        }
    }

    fn translate_sig(&self, symbol: &monoir::Symbol) -> Result<codegen::ir::Signature> {
        if let monoir::Type::Function {
            params_ty,
            return_ty,
        } = &symbol.ty
        {
            let mut sig = self.inner.make_signature();
            for param in params_ty {
                let ty = self.translate_type(param);
                let param = codegen::ir::AbiParam::new(ty);
                sig.params.push(param);
            }

            if **return_ty != monoir::Type::Unit {
                let ty = self.translate_type(&return_ty);
                let param = codegen::ir::AbiParam::new(ty);
                sig.returns.push(param);
            }
            Ok(sig)
        } else {
            Err(Error::new(format!("{symbol:?} is not a function type")))
        }
    }

    fn declare_func(
        &mut self,
        symbol: &monoir::Symbol,
        linkage: cranelift_module::Linkage,
        sig: codegen::ir::Signature,
    ) -> Result<FuncId> {
        self
            .inner
            .declare_function(&symbol.name, linkage, &sig)
            .map_err(|e| Error::new(format!(" Error {e}")))
    }

    pub(super) fn setup_params(
        &self,
        params: &[monoir::Symbol],
        builder: &mut FunctionBuilder<'_>,
        block: cranelift::prelude::Block,
    ) -> Vec<Variable> {
        let mut vars = Vec::new();
        for (i, param) in params.iter().enumerate() {
            // TODO: cranelift_frontend should really have an API to make it
            // easy to set up param variables.
            let val = builder.block_params(block)[i];
            let var = self.declare_variable(&param, builder);
            builder.def_var(var, val);
            vars.push(var);
        }
        vars
    }

    pub(super) fn create_entry_block(
        &self,
        builder: &mut FunctionBuilder<'_>,
    ) -> cranelift::prelude::Block {
        // Create the entry block, to start emitting code in.
        let entry_block = builder.create_block();

        // Since this is the entry block, add block parameters corresponding to
        // the function's parameters.
        //
        // TODO: Streamline the API here.
        builder.append_block_params_for_function_params(entry_block);

        // Tell the builder to emit code in this block.
        builder.switch_to_block(entry_block);

        // And, tell the builder that this block will have no further
        // predecessors. Since it's the entry block, it won't have any
        // predecessors.
        builder.seal_block(entry_block);

        entry_block
    }
}
