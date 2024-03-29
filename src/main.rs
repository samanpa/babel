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

    let mod_name = filenm.file_stem().unwrap().to_str().unwrap().to_string();

    let rename = Rename::new();
    let typecheck = TypeChecker::new();
    let specialize = Specialize::new();
    let lambda_lift = LambdaLift::new();
    let simplify = Simplify::new();
    let codegen = CodeGen::new();
    let link = Link::new(mod_name.clone());

    let parser = babel::parser::ModuleParser::new();
    let module = parser
        .parse(&mod_name, &file_contents)
        .map_err(|lalr_err| babel::Error::new(format!("{:?}", lalr_err)))?;

    let modules = vec![module];

    let _ = passes![
        modules
        => rename
        => typecheck
        => specialize
        => lambda_lift
        => simplify
        => codegen
        => link
    ];

    Ok(())
}

fn main() {
    if env::args().len() == 1 {
        println!("No filename provided");
        std::process::exit(2);
    }

    let file_name = env::args().nth(1).unwrap();
    let file_name = Path::new(&file_name);
    let file = File::open(file_name).unwrap();

    match compile(file, file_name) {
        Ok(()) => (),
        Err(e) => {
            println!("ERROR: {}", e);
            std::process::exit(1);
        }
    }
}
