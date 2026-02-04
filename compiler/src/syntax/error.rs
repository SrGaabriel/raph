extern crate alloc;

use crate::syntax::SourceSpan;
use crate::syntax::token::TokenKind;
use alloc::vec::Vec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LexError<'a> {
    pub kind: LexErrorKind,
    pub span: SourceSpan,
    pub lexeme: &'a [u8],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LexErrorKind {
    UnexpectedEndOfInput,
    InvalidToken,
    UnterminatedString,
    UnexpectedChar(char),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub span: SourceSpan,
    pub expected: Vec<TokenKind>,
    pub found: Option<TokenKind>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseErrorKind {
    UnexpectedToken,
    UnexpectedEndOfInput,
    UnclosedDelimiter {
        open: TokenKind,
        expected_close: TokenKind,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyntacticError<'a> {
    Lex(LexError<'a>),
    Parse(ParseError),
}

impl<'a> From<LexError<'a>> for SyntacticError<'a> {
    fn from(err: LexError<'a>) -> Self {
        SyntacticError::Lex(err)
    }
}

impl From<ParseError> for SyntacticError<'_> {
    fn from(err: ParseError) -> Self {
        SyntacticError::Parse(err)
    }
}
