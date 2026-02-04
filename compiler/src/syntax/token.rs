use crate::syntax::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token<'a> {
    pub lexeme: &'a [u8],
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Identifier,
    Number,
    String,
    Equal,
    Struct,
    Comma,
    Colon,
    LBrace,
    RBrace,
    LParen,
    RParen,
    Semicolon,
    EndOfFile,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenStream<'a> {
    pub tokens: &'a [Token<'a>],
    pub position: usize,
}
