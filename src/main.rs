#[macro_use(passes)]
extern crate babel;
use std::env;
use std::fs::File;
use std::path::Path;

fn compile(file: File, filenm: &Path) -> babel::Result<()> {
    use std::io::Read;
    
    let mut file_contents = babel::prelude::PRELUDE.to_string();
    let mut file = file;
    let _ = file.read_to_string(&mut file_contents);

    let mod_name     = filenm.file_stem()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    let rename       = babel::rename::Rename::new();
    let typecheck    = babel::typing::TypeChecker::new();
    let monomorphize = babel::specialize::Specialize::new();
    let uncurry      = babel::uncurry::Uncurry::new();
    let codegen      = babel::codegen::CodeGen::new(mod_name.clone());
    let link         = babel::link::Link::new(mod_name.clone());
    
    use babel::Pass;
    let mut module  = babel::parser::parse_Module(&file_contents)?;
    module.set_name(mod_name);
    let modules = vec![module];
    
    let _ = passes![
        modules
        => rename
        => typecheck
        => monomorphize
        => uncurry
        => codegen
        => link    
    ];

    Ok(())
}

fn main() {
    let file_name = env::args().nth(1).unwrap();
    let file_name = Path::new(&file_name);
    let file = File::open(file_name).unwrap();
    
    use std::error::Error;
    match compile(file, file_name) {
        Ok(()) => (),
        Err(e) => println!("ERROR: {}", e.description())
    }
}
