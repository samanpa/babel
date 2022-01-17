use std::borrow::Borrow;

use crate::monoir;
use crate::{Error, Result, Vector};
use cranelift_object::{ObjectBuilder, ObjectModule};

pub struct CodeGen {
    module: ObjectModule,
    output_file: String,
}

impl crate::Pass for CodeGen {
    type Input = Vec<monoir::Module>;
    type Output = Vec<String>;

    fn run(mut self, modules: Self::Input) -> Result<Self::Output> {
        Vector::map(&modules, |module| self.codegen_module(&module))
    }
}

impl CodeGen {
    pub fn new(output_file: String) -> Result<Self> {
        use cranelift::codegen::{self, settings};
        let triple = target_lexicon::Triple::host();
        let flags = settings::Flags::new(settings::builder());
        let target_isa = codegen::isa::lookup(triple.clone())
            .map_err(|e| Error::new(format!("Unsupported triple {e:?}")))?
            .finish(flags);
        let builder = ObjectBuilder::new(
            target_isa,
            output_file.to_string(),
            cranelift_module::default_libcall_names(),
        )
            .map_err(|e| Error::new(format!("Cannot create cranelift module")))?;
	let module = ObjectModule::new(builder);

        Ok(Self {
            output_file,
            module,
        })
    }

    fn codegen_module(&mut self, module: &monoir::Module) -> Result<String> {
        Ok(String::new())
    }
}
