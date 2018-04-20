pub use rename::Rename;
pub use typing::TypeChecker;
pub use specialize::Specialize;
pub use uncurry::Uncurry;
pub use codegen::CodeGen;
pub use lambda_lift::LambdaLift;
pub use link::Link;    

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
