pub mod ast;
pub mod parser;
pub mod rename;
pub mod ir;
pub mod translate;
pub (crate) mod scoped_map;

#[derive(Debug)]
pub struct Error{
    msg: String,
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        Ok(())
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        &self.msg
    }

    fn cause(&self) -> Option<&std::error::Error> {
        None
    }
}

impl Error {
    pub fn new<T>(msg: T) -> Self where T: Into<String>{
        Error{ msg: msg.into() }
    }
}

extern crate lalrpop_util;
impl <'a> From<lalrpop_util::ParseError<usize, (usize, &'a str), ()>>for Error {
    fn from(f: lalrpop_util::ParseError<usize, (usize, &'a str), ()>) -> Self {
        Self{ msg: format!("{:?}", f)}
    }
    
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait Pass {
    type Input;
    type Output;

    fn run(&mut self, source: Self::Input) -> Result<Self::Output>;
}
