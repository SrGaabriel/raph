extern crate alloc;

use alloc::vec::Vec;
use chumsky::{input::Input as _, prelude::*};

use crate::syntax::{
    SourceSpan,
    error::{ParseError, ParseErrorKind},
    token::TokenKind,
};

impl chumsky::span::Span for SourceSpan {
    type Offset = usize;
    type Context = usize;

    fn new(context: Self::Context, range: core::ops::Range<Self::Offset>) -> Self {
        use crate::syntax::SourcePos;
        SourceSpan {
            start: SourcePos {
                file_index: context,
                line: 0,
                column: 0,
                byte_offset: range.start,
            },
            end: SourcePos {
                file_index: context,
                line: 0,
                column: 0,
                byte_offset: range.end,
            },
        }
    }

    fn context(&self) -> Self::Context {
        self.start.file_index
    }

    fn start(&self) -> Self::Offset {
        self.start.byte_offset
    }

    fn end(&self) -> Self::Offset {
        self.end.byte_offset
    }
}

type TokenInput<'a> =
    chumsky::input::MappedInput<'a, TokenKind, SourceSpan, &'a [(TokenKind, SourceSpan)]>;

type ParserExtra<'a> = extra::Err<Rich<'a, TokenKind, SourceSpan>>;

fn just_token<'a>(
    kind: TokenKind,
) -> impl Parser<'a, TokenInput<'a>, TokenKind, ParserExtra<'a>> + Clone {
    any().filter(move |t: &TokenKind| *t == kind)
}

pub fn parse(
    tokens: &[(TokenKind, SourceSpan)],
    eoi_span: SourceSpan,
) -> (Option<()>, Vec<ParseError>) {
    let input = tokens.split_token_span(eoi_span);

    let parser = program();

    let (output, errors) = parser.parse(input).into_output_errors();
    let errors = errors.into_iter().map(rich_to_parse_error).collect();

    (output, errors)
}

fn program<'a>() -> impl Parser<'a, TokenInput<'a>, (), ParserExtra<'a>> {
    end()
}

fn rich_to_parse_error(err: Rich<'_, TokenKind, SourceSpan>) -> ParseError {
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
