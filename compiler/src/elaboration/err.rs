use alloc::string::String;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ElabError {
    #[error("expected root")]
    ExpectedRoot,
    #[error("undefined variable `{0}`")]
    UndefinedVariable(String),
    #[error("undefined constructor `{0}`")]
    UndefinedConstructor(String),
    #[error("type mismatch: expected `{expected}`, found `{found}`")]
    TypeMismatch { expected: crate::spine::Term, found: crate::spine::Term },
    #[error("unsupported syntax: `{0:?}`")]
    UnsupportedSyntax(crate::syntax::tree::SyntaxExpr),
    #[error("can't apply to non-function type `{0}`")]
    NotAFunction(crate::spine::Term),
}