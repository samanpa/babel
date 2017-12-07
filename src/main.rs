#[macro_use(passes)]
extern crate babel;
use std::env;
use std::fs::File;
    
fn compile(file: File) -> babel::Result<()> {
    use std::io::Read;
    
    let mut file_contents = babel::prelude::PRELUDE.to_string();
    let mut file = file;
    let _ = file.read_to_string(&mut file_contents);

    
    let rename       = babel::rename::Rename::new();
    let typecheck    = babel::typing::SimpleTypeChecker::new();
    let monomorphise = babel::monomorphise::Monomorphise::new();
    let elaborate    = babel::elaborate::Elaborate::new("module".to_string());
    let codegen      = babel::codegen::CodeGen::new("module.o".to_string());

    use babel::Pass;
    let asts  = vec![babel::parser::parse_TopLevel(&file_contents)?];
    let _ = passes![
        asts
        => rename
        => typecheck
        => monomorphise
        => elaborate
        => codegen
    ];
    
    Ok(())
}

fn main() {
    let file = env::args().nth(1).unwrap();
    let file = File::open(file).unwrap();
    
    use std::error::Error;
    match compile(file) {
        Ok(()) => (),
        Err(e) => println!("ERROR: {}", e.description())
    }
}
