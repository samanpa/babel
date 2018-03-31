#[macro_use(passes)]
extern crate babel;
use std::env;
use std::fs::File;
use std::path::Path;

use babel::passes::*;

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

    let rename     = Rename::new();
    let typecheck  = TypeChecker::new();
    let specialize = Specialize::new();
    let uncurry    = Uncurry::new();
    let codegen    = CodeGen::new(mod_name.clone());
    let link       = Link::new(mod_name.clone());

    let parser     = babel::parser::ModuleParser::new();
    let mut module = parser.parse(&file_contents)
        .map_err( |lalr_err| babel::Error::new(format!("{:?}", lalr_err) ))?;
    
    module.set_name(mod_name);
    let modules = vec![module];

    let _ = passes![
        modules
        => rename
        => typecheck
        => specialize
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
        Err(e) => {
            println!("ERROR: {}", e.description());
            std::process::exit(1);
        }
    }
}
