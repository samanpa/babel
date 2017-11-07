extern crate babel;

use std::env;
use std::fs::File;
    
fn compile(file: File) -> babel::Result<()> {
    use std::io::Read;
    
    let mut file_contents = String::new();
    let mut file = file;
    let _ = file.read_to_string(&mut file_contents);

    
    let rename    = babel::rename::Rename::new();
    let typecheck = babel::typing::SimpleTypeChecker::new();
    let translate = babel::translate::Translate::new();
    let codegen   = babel::codegen::CodeGen::new();

    use babel::Pass;
    let ast  = babel::parser::parse_TopLevel(&file_contents)?;
    let asts = vec![ast];
    let asts = rename.run(asts)?;
    let asts = typecheck.run(asts)?;
    let ir   = translate.run(asts)?;
    let _    = codegen.run(ir)?;
    
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
