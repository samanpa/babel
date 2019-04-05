pub use codegen::CodeGen;
pub use lambda_lift::LambdaLift;
pub use link::Link;
pub use rename::Rename;
pub use simplify::Simplify;
pub use specialize::Specialize;
pub use typecheck::TypeChecker;

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
