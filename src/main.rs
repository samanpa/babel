extern crate babel;

use std::env;
use std::fs::File;
    
fn main() {
    use std::io::Read;
    
    let mut file_contents = String::new();
    let file = env::args().nth(1).unwrap();
    let mut file = File::open(file).unwrap();
    let _ = file.read_to_string(&mut file_contents);

    let ast  = babel::parser::parse_TopLevel(&file_contents);
    println!("{:?}", ast);
}
