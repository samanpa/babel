use crate::monoir;
use crate::{Error, Result, Vector};

mod prelude;
mod translate;

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
        let cranelift_module = translate::Translator::new(&name)?;
        let module = cranelift_module.translate(module)?;
        let bytes = module.finish().emit().unwrap();
        std::fs::write(format!("{name}.o"), bytes).unwrap();
        Err(Error::new("Cranelift codegen not complete".to_string()))
    }
}
