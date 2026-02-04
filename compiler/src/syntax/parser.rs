extern crate alloc;

use alloc::vec::Vec;
use chumsky::{input::Input as _, prelude::*};

use crate::syntax::{
    Span,
    error::{ParseError, ParseErrorKind},
    token::TokenKind,
};

impl chumsky::span::Span for Span {
    type Offset = usize;
    type Context = usize; // file index

    fn new(context: Self::Context, range: core::ops::Range<Self::Offset>) -> Self {
        Span {
            file: context,
            start: range.start,
            end: range.end,
        }
    }

    fn context(&self) -> Self::Context {
        self.file
    }

    fn start(&self) -> Self::Offset {
        self.start
    }

    fn end(&self) -> Self::Offset {
        self.end
    }
}

type TokenInput<'a> = chumsky::input::MappedInput<'a, TokenKind, Span, &'a [(TokenKind, Span)]>;

type ParserExtra<'a> = extra::Err<Rich<'a, TokenKind, Span>>;

fn just_token<'a>(
    kind: TokenKind,
) -> impl Parser<'a, TokenInput<'a>, TokenKind, ParserExtra<'a>> + Clone {
    any().filter(move |t: &TokenKind| *t == kind)
}

pub fn parse(tokens: &[(TokenKind, Span)], eoi_span: Span) -> (Option<()>, Vec<ParseError>) {
    let input = tokens.split_token_span(eoi_span);

    let parser = program();

    let (output, errors) = parser.parse(input).into_output_errors();
    let errors = errors.into_iter().map(rich_to_parse_error).collect();

    (output, errors)
}

fn program<'a>() -> impl Parser<'a, TokenInput<'a>, (), ParserExtra<'a>> {
    end()
}

fn rich_to_parse_error(err: Rich<'_, TokenKind, Span>) -> ParseError {
    let span = *err.span();
    let found = err.found().copied();
    let expected: Vec<TokenKind> = err
        .expected()
        .filter_map(|e| match e {
            chumsky::error::RichPattern::Token(t) => Some(t.into_inner().clone()),
            chumsky::error::RichPattern::Label(_) => None,
            chumsky::error::RichPattern::EndOfInput => None,
            _ => None,
        })
        .collect();

    ParseError {
        kind: if found.is_none() {
            ParseErrorKind::UnexpectedEndOfInput
        } else {
            ParseErrorKind::UnexpectedToken
        },
        span,
        expected,
        found,
    }
}
