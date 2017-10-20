extern crate babel;

use std::env;
use std::fs::File;
    
fn compile(file: File) -> babel::Result<()> {
    use std::io::Read;
    
    let mut file_contents = String::new();
    let mut file = file;
    let _ = file.read_to_string(&mut file_contents);

    let mut desugar  = babel::desugar::Desugar{};

    use babel::Pass;
    let ast  = babel::parser::parse_TopLevel(&file_contents)?;
    let asts = vec![ast];
    let ir   = desugar.run(asts)?;
    
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
