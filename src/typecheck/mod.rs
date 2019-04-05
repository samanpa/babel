mod env;
mod hm;
mod typecheck;
mod unify;

type Type = ::types::Type<::types::TyVar>;
type ForAll = ::types::ForAll<::types::TyVar>;

pub use self::typecheck::TypeChecker;
