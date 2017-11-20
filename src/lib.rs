pub mod ast;
pub mod prelude;
pub mod parser;
pub mod hir;
pub mod rename;
pub mod ir;
pub mod monomorphise;
pub mod elaborate;
pub mod codegen;
pub mod typing;
pub (crate) mod scoped_map;

#[derive(Debug)]
pub struct Error{
    msg: String,
}

impl std::fmt::Display for Error {
    fn fmt(&self, _fmt: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        //fmt.print()?;
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

    fn run(self, source: Self::Input) -> ::Result<Self::Output>;
}

pub struct VecUtil {}

impl VecUtil {
    pub fn map<I,O,F>(v: &Vec<I>, mut f: F) -> Result<Vec<O>> 
        where F: FnMut(&I) -> Result<O>{
        let mut res = Vec::new();
        for val in v {
            let val = f(val)?;
            res.push(val);
        }
        Ok(res)
    }
    pub fn mapt<I,O,F>(v: Vec<I>, mut f: F) -> Result<Vec<O>> 
        where F: FnMut(I) -> Result<O>{
        let mut res = Vec::new();
        for val in v {
            let val = f(val)?;
            res.push(val);
        }
        Ok(res)
    }
}

#[macro_export]
macro_rules! passes {
    ( $expr:expr => $($pass:expr) => + ) => {
        {
            let ret = $expr;
            $(
                let ret = Pass::run($pass, ret)?;
            )*
            ret
        }
    };
}
