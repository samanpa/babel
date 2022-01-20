use crate::monoir;
use crate::{Result, Vector};

mod expr;
mod intrinsics;
mod module;

pub struct CodeGen {}

impl crate::Pass for CodeGen {
    type Input = Vec<monoir::Module>;
    type Output = Vec<String>;

    fn run(mut self, modules: Self::Input) -> Result<Self::Output> {
        Vector::mapt(modules, |v| Self::codegen_module(&mut self, v))
    }
}

impl CodeGen {
    pub fn new() -> Self {
        Self {}
    }

    fn codegen_module(&mut self, module: monoir::Module) -> Result<String> {
        let name = module.name.to_string();
        let cranelift_module = module::Translator::new(&name)?;
        let module = cranelift_module.translate(module)?;
        let bytes = module.finish().emit().unwrap();
        let object_file = format!("{name}.o");
        std::fs::write(&object_file, bytes).unwrap();
        Ok(object_file)
    }
}
