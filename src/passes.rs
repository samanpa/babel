pub use crate::codegen::CodeGen;
pub use crate::lambda_lift::LambdaLift;
pub use crate::link::Link;
pub use crate::rename::Rename;
pub use crate::simplify::Simplify;
pub use crate::specialize::Specialize;
pub use crate::typecheck::TypeChecker;

#[macro_export]
macro_rules! passes {
    ( $expr:expr => $($pass:expr) => + ) => {
        {
            let ret = $expr;
            $(
                let ret = babel::Pass::run($pass, ret)?;
            )*
            ret
        }
    };
}
