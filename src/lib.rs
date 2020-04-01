#[macro_use]
extern crate lalrpop_util;

pub mod ast;
pub mod prelude;
lalrpop_mod!(pub parser);
pub mod codegen;
pub mod idtree;
pub mod lambda_lift;
pub mod link;
pub mod monoir;
pub mod passes;
pub mod rename;
pub(crate) mod scoped_map;
pub mod simplify;
pub mod specialize;
pub mod typecheck;
pub mod types;
pub mod utils;
pub mod xir;

#[derive(Debug)]
pub struct Error {
    msg: String,
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
	write!(fmt, "{}", self.msg)?;
        Ok(())
    }
}

impl std::error::Error for Error {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }
}

impl Error {
    pub fn new<T>(msg: T) -> Self
    where
        T: Into<String>,
    {
        Error { msg: msg.into() }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait Pass {
    type Input;
    type Output;

    fn run(self, source: Self::Input) -> crate::Result<Self::Output>;
}

pub struct Vector {}

impl Vector {
    pub fn map<I, O, F>(v: &[I], mut f: F) -> Result<Vec<O>>
    where
        F: FnMut(&I) -> Result<O>,
    {
        let mut res = Vec::with_capacity(v.len());
        for val in v {
            let val = f(val)?;
            res.push(val);
        }
        Ok(res)
    }
    pub fn mapt<I, O, F>(v: Vec<I>, mut f: F) -> Result<Vec<O>>
    where
        F: FnMut(I) -> Result<O>,
    {
        let mut res = Vec::with_capacity(v.len());
        for val in v {
            let val = f(val)?;
            res.push(val);
        }
        Ok(res)
    }
}

static mut COUNT: u32 = 0;
pub fn fresh_id() -> u32 {
    unsafe {
        COUNT += 1;
        COUNT
    }
}
