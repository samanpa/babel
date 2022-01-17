use lalrpop_util::lalrpop_mod;

pub mod ast;
mod error;
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

pub use error::Error;
pub type Result<T> = std::result::Result<T, Error>;

pub trait Pass {
    type Input;
    type Output;

    fn run(self, source: Self::Input) -> crate::Result<Self::Output>;
}

pub struct Vector {}

impl Vector {
    pub fn fmap<I, O>(v: impl Iterator<Item = I>, mut f: impl FnMut(I) -> O) -> Vec<O> {
        let mut res = Vec::with_capacity(v.size_hint().0);
        for val in v {
            res.push(f(val));
        }
        res
    }

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
