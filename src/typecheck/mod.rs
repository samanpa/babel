mod env;
mod hm;
mod typecheck;
mod unify;

type Type = crate::types::Type<crate::types::TyVar>;
type ForAll = crate::types::ForAll<crate::types::TyVar>;

pub use self::typecheck::TypeChecker;
