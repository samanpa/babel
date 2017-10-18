extern crate babel_parsing;

use std::env;
use std::fs::File;
    
fn main() {
    use std::io::Read;
    
    let mut file_contents = String::new();
    let file = env::args().nth(1).unwrap();
    let mut file = File::open(file).unwrap();
    file.read_to_string(&mut file_contents);

    println!("{}", file_contents);
    
    let ast  = babel_parsing::parser::parse_TopLevel(&file_contents);
    println!("{:?}", ast);
}
