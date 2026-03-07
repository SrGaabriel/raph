extern crate alloc;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

use crate::{
    spine::Literal,
    syntax::{
        Span,
        error::{ParseError, ParseErrorKind},
        token::{Token, TokenKind},
        tree::{
            DefBody, InductiveConstructor, InfixOp, InstanceMember, PatternMatchArm, RecordField,
            SyntaxAttribute, SyntaxBinder, SyntaxExpr as Expr, SyntaxPattern, SyntaxTree,
            SyntaxTreeDeclaration as Decl,
        },
    },
};

struct Parser<'a> {
    tokens: &'a [Token<'a>],
    pos: usize,
    file: usize,
    errors: Vec<ParseError>,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [Token<'a>], file: usize) -> Self {
        Self {
            tokens,
            pos: 0,
            file,
            errors: Vec::new(),
        }
    }

    fn peek(&self) -> TokenKind {
        if self.pos < self.tokens.len() {
            self.tokens[self.pos].kind
        } else {
            TokenKind::EndOfFile
        }
    }

    fn current(&self) -> Option<&Token<'a>> {
        self.tokens.get(self.pos)
    }

    fn current_span(&self) -> Span {
        if let Some(t) = self.current() {
            t.span
        } else {
            Span::empty(self.file, self.tokens.last().map_or(0, |t| t.span.end))
        }
    }

    fn at(&self, kind: TokenKind) -> bool {
        self.peek() == kind
    }

    fn eat(&mut self, kind: TokenKind) -> Option<Token<'a>> {
        if self.at(kind) {
            let tok = self.tokens[self.pos];
            self.pos += 1;
            Some(tok)
        } else {
            None
        }
    }

    fn expect(&mut self, kind: TokenKind) -> Token<'a> {
        if let Some(tok) = self.eat(kind) {
            tok
        } else {
            let span = self.current_span();
            let found = if self.pos < self.tokens.len() {
                Some(self.tokens[self.pos].kind)
            } else {
                None
            };
            self.errors.push(ParseError {
                kind: if found.is_some() {
                    ParseErrorKind::UnexpectedToken
                } else {
                    ParseErrorKind::UnexpectedEndOfInput
                },
                span,
                expected: alloc::vec![kind],
                found,
            });
            Token {
                kind,
                lexeme: b"",
                span,
            }
        }
    }

    fn bump(&mut self) -> Token<'a> {
        let tok = self.tokens[self.pos];
        self.pos += 1;
        tok
    }

}

fn lexeme_to_string(lexeme: &[u8]) -> String {
    String::from_utf8_lossy(lexeme).into_owned()
}

pub fn parse<'a>(
    tokens: &'a [(Token<'a>, Span)],
    _eoi_span: Span,
) -> (Option<SyntaxTree>, Vec<ParseError>) {
    let plain: Vec<Token<'a>> = tokens.iter().map(|(t, _)| *t).collect();
    let file = tokens.first().map_or(0, |(t, _)| t.span.file);
    let mut p = Parser::new(&plain, file);
    let tree = parse_source_file(&mut p);
    if p.errors.is_empty() {
        (Some(tree), Vec::new())
    } else {
        (Some(tree), p.errors)
    }
}


fn parse_source_file(p: &mut Parser) -> SyntaxTree {
    let mut declarations = Vec::new();
    while !p.at(TokenKind::EndOfFile) {
        if let Some(decl) = parse_declaration(p) {
            declarations.push(decl);
        } else {
            if !p.at(TokenKind::EndOfFile) {
                let tok = p.bump();
                p.errors.push(ParseError {
                    kind: ParseErrorKind::UnexpectedToken,
                    span: tok.span,
                    expected: Vec::new(),
                    found: Some(tok.kind),
                });
            }
        }
    }
    SyntaxTree { declarations }
}

fn parse_declaration(p: &mut Parser) -> Option<Decl> {
    let attrs = parse_attributes(p);

    match p.peek() {
        TokenKind::Def | TokenKind::Theorem => Some(parse_def(p)),
        TokenKind::Eval => Some(parse_eval(p)),
        TokenKind::Record => Some(parse_record(p, attrs)),
        TokenKind::Extern => Some(parse_extern(p)),
        TokenKind::Inductive => Some(parse_inductive(p, attrs)),
        TokenKind::Class => Some(parse_class(p, attrs)),
        TokenKind::Instance => Some(parse_instance(p)),
        TokenKind::Alias => Some(parse_alias(p)),
        _ => {
            if !attrs.is_empty() {
                let span = attrs[0].span;
                p.errors.push(ParseError {
                    kind: ParseErrorKind::UnexpectedToken,
                    span,
                    expected: Vec::new(),
                    found: Some(p.peek()),
                });
            }
            None
        }
    }
}


fn parse_attributes(p: &mut Parser) -> Vec<SyntaxAttribute> {
    let mut attrs = Vec::new();
    while p.at(TokenKind::At) {
        let at = p.bump();
        p.expect(TokenKind::LBracket);
        let name_tok = p.expect(TokenKind::LowerIdentifier);
        let mut args = Vec::new();
        while !p.at(TokenKind::RBracket) && !p.at(TokenKind::EndOfFile) {
            args.push(parse_expr_atom(p));
        }
        let rbracket = p.expect(TokenKind::RBracket);
        attrs.push(SyntaxAttribute {
            span: Span::new(at.span.file, at.span.start, rbracket.span.end),
            name: lexeme_to_string(name_tok.lexeme),
            args,
        });
    }
    attrs
}

fn parse_def(p: &mut Parser) -> Decl {
    let kw = p.bump(); // todo: separate theorem
    let name_tok = p.expect(TokenKind::LowerIdentifier);
    let binders = parse_binders(p);
    p.expect(TokenKind::Colon);
    let return_type = parse_expr(p);
    let body = parse_def_body(p);
    let span = Span::new(kw.span.file, kw.span.start, body_end(&body));
    Decl::Def {
        span,
        name: lexeme_to_string(name_tok.lexeme),
        binders,
        return_type,
        body,
    }
}

fn body_end(body: &DefBody) -> usize {
    match body {
        DefBody::Expr(e) => {
            use crate::syntax::Spanned;
            e.span().end
        }
        DefBody::PatternMatch { span, .. } => span.end,
    }
}

fn parse_def_body(p: &mut Parser) -> DefBody {
    if p.at(TokenKind::Equal) {
        p.bump();
        DefBody::Expr(parse_expr(p))
    } else if p.at(TokenKind::Pipe) {
        let arms = parse_pattern_match_arms(p);
        let span = if arms.is_empty() {
            p.current_span()
        } else {
            Span::new(
                arms[0].span.file,
                arms[0].span.start,
                arms.last().unwrap().span.end,
            )
        };
        DefBody::PatternMatch { arms, span }
    } else {
        let span = p.current_span();
        p.errors.push(ParseError {
            kind: ParseErrorKind::UnexpectedToken,
            span,
            expected: alloc::vec![TokenKind::Equal, TokenKind::Pipe],
            found: Some(p.peek()),
        });
        DefBody::Expr(Expr::Hole(span))
    }
}

fn parse_pattern_match_arms(p: &mut Parser) -> Vec<PatternMatchArm> {
    let mut arms = Vec::new();
    while p.at(TokenKind::Pipe) {
        p.bump();
        let patterns = parse_comma_separated_patterns(p);
        p.expect(TokenKind::FatArrow);
        let rhs = parse_expr(p);
        let span = if patterns.is_empty() {
            use crate::syntax::Spanned;
            rhs.span()
        } else {
            use crate::syntax::Spanned;
            Span::new(
                patterns[0].span().file,
                patterns[0].span().start,
                rhs.span().end,
            )
        };
        arms.push(PatternMatchArm {
            span,
            patterns,
            rhs: Box::new(rhs),
        });
    }
    arms
}

fn parse_comma_separated_patterns(p: &mut Parser) -> Vec<SyntaxPattern> {
    let mut pats = Vec::new();
    pats.push(parse_pattern(p));
    while p.at(TokenKind::Comma) {
        p.bump();
        pats.push(parse_pattern(p));
    }
    pats
}

fn parse_eval(p: &mut Parser) -> Decl {
    let kw = p.bump();
    let expr = parse_expr(p);
    let semi = p.expect(TokenKind::Semicolon);
    Decl::Eval {
        span: Span::new(kw.span.file, kw.span.start, semi.span.end),
        expr,
    }
}

fn parse_record(p: &mut Parser, attributes: Vec<SyntaxAttribute>) -> Decl {
    let kw = p.bump();
    let name_tok = p.expect(TokenKind::UpperIdentifier);
    let binders = parse_binders(p);
    p.expect(TokenKind::LBrace);
    let fields = parse_record_fields(p);
    p.eat(TokenKind::Comma);
    let rbrace = p.expect(TokenKind::RBrace);
    Decl::Record {
        span: Span::new(kw.span.file, kw.span.start, rbrace.span.end),
        attributes,
        name: lexeme_to_string(name_tok.lexeme),
        binders,
        fields,
    }
}

fn parse_record_fields(p: &mut Parser) -> Vec<RecordField> {
    let mut fields = Vec::new();
    while p.at(TokenKind::LowerIdentifier) || p.at(TokenKind::At) {
        let attrs = parse_attributes(p);
        let name_tok = p.expect(TokenKind::LowerIdentifier);
        p.expect(TokenKind::Colon);
        let ty = parse_expr(p);
        use crate::syntax::Spanned;
        let span = Span::new(name_tok.span.file, name_tok.span.start, ty.span().end);
        fields.push(RecordField {
            span,
            attributes: attrs,
            name: lexeme_to_string(name_tok.lexeme),
            type_ann: Box::new(ty),
        });
        if !p.at(TokenKind::Comma) {
            break;
        }
        p.bump();
    }
    fields
}

fn parse_extern(p: &mut Parser) -> Decl {
    let kw = p.bump();
    let name_tok = p.expect(TokenKind::LowerIdentifier);
    p.expect(TokenKind::Colon);
    let type_ann = parse_expr(p);
    use crate::syntax::Spanned;
    Decl::Extern {
        span: Span::new(kw.span.file, kw.span.start, type_ann.span().end),
        name: lexeme_to_string(name_tok.lexeme),
        type_ann,
    }
}

fn parse_inductive(p: &mut Parser, attributes: Vec<SyntaxAttribute>) -> Decl {
    let kw = p.bump();
    let name_tok = p.expect(TokenKind::UpperIdentifier);
    let binders = parse_binders(p);
    let index_type = if p.at(TokenKind::Colon) {
        p.bump();
        Some(parse_expr(p))
    } else {
        None
    };
    p.expect(TokenKind::LBrace);
    let constructors = parse_inductive_constructors(p);
    p.eat(TokenKind::Comma);
    let rbrace = p.expect(TokenKind::RBrace);
    Decl::Inductive {
        span: Span::new(kw.span.file, kw.span.start, rbrace.span.end),
        attributes,
        name: lexeme_to_string(name_tok.lexeme),
        index_type,
        binders,
        constructors,
    }
}

fn parse_inductive_constructors(p: &mut Parser) -> Vec<InductiveConstructor> {
    let mut ctors = Vec::new();
    while p.at(TokenKind::LowerIdentifier) {
        let name_tok = p.bump();
        let binders = parse_binders(p);
        let type_ann = if p.at(TokenKind::Colon) {
            p.bump();
            Some(parse_expr(p))
        } else {
            None
        };
        ctors.push(InductiveConstructor {
            span: name_tok.span,
            name: lexeme_to_string(name_tok.lexeme),
            binders,
            type_ann,
        });
        if !p.at(TokenKind::Comma) {
            break;
        }
        p.bump();
    }
    ctors
}

fn parse_class(p: &mut Parser, attributes: Vec<SyntaxAttribute>) -> Decl {
    let kw = p.bump();
    let name_tok = p.expect(TokenKind::UpperIdentifier);
    let binders = parse_binders(p);
    p.expect(TokenKind::LBrace);
    let members = parse_record_fields(p);
    p.eat(TokenKind::Comma);
    let rbrace = p.expect(TokenKind::RBrace);
    Decl::Class {
        attributes,
        span: Span::new(kw.span.file, kw.span.start, rbrace.span.end),
        name: lexeme_to_string(name_tok.lexeme),
        binders,
        members,
    }
}

fn parse_instance(p: &mut Parser) -> Decl {
    let kw = p.bump();
    let name_tok = p.expect(TokenKind::LowerIdentifier);
    let binders = parse_binders(p);
    p.expect(TokenKind::Colon);
    let type_ann = parse_expr(p);
    p.expect(TokenKind::LBrace);
    let members = parse_instance_members(p);
    p.eat(TokenKind::Comma);
    let rbrace = p.expect(TokenKind::RBrace);
    Decl::Instance {
        span: Span::new(kw.span.file, kw.span.start, rbrace.span.end),
        name: lexeme_to_string(name_tok.lexeme),
        binders,
        type_ann,
        members,
    }
}

fn parse_instance_members(p: &mut Parser) -> Vec<InstanceMember> {
    let mut members = Vec::new();
    while p.at(TokenKind::LowerIdentifier) {
        let name_tok = p.bump();
        p.expect(TokenKind::Equal);
        let value = parse_expr(p);
        use crate::syntax::Spanned;
        members.push(InstanceMember {
            span: Span::new(name_tok.span.file, name_tok.span.start, value.span().end),
            name: lexeme_to_string(name_tok.lexeme),
            value,
        });
        if !p.at(TokenKind::Comma) {
            break;
        }
        p.bump();
    }
    members
}

fn parse_alias(p: &mut Parser) -> Decl {
    p.bump();
    let name_tok = p.expect(TokenKind::UpperIdentifier);
    let binders = parse_binders(p);
    let type_ann = if p.at(TokenKind::Colon) {
        p.bump();
        Some(parse_expr(p))
    } else {
        None
    };
    p.expect(TokenKind::Equal);
    let value = parse_expr(p);
    use crate::syntax::Spanned;
    Decl::Alias {
        span: Span::new(name_tok.span.file, name_tok.span.start, value.span().end),
        name: lexeme_to_string(name_tok.lexeme),
        binders,
        type_ann,
        value,
    }
}

fn parse_binders(p: &mut Parser) -> Vec<SyntaxBinder> {
    let mut binders = Vec::new();
    while let Some(b) = try_parse_binder(p) {
        binders.push(b);
    }
    binders
}

fn try_parse_binder(p: &mut Parser) -> Option<SyntaxBinder> {
    match p.peek() {
        TokenKind::LParen => try_parse_delimited_binder(p, TokenKind::LParen, TokenKind::RParen, |span, name, ty| {
            SyntaxBinder::Explicit(span, name, ty)
        }),
        TokenKind::LBrace => try_parse_delimited_binder(p, TokenKind::LBrace, TokenKind::RBrace, |span, name, ty| {
            SyntaxBinder::Implicit(span, name, ty)
        }),
        TokenKind::LBracket => try_parse_delimited_binder(p, TokenKind::LBracket, TokenKind::RBracket, |span, name, ty| {
            SyntaxBinder::Instance(span, name, ty)
        }),
        _ => None,
    }
}

fn try_parse_delimited_binder(
    p: &mut Parser,
    open: TokenKind,
    close: TokenKind,
    mk: fn(Span, String, Box<Expr>) -> SyntaxBinder,
) -> Option<SyntaxBinder> {
    if p.pos + 2 < p.tokens.len()
        && p.tokens[p.pos].kind == open
        && p.tokens[p.pos + 1].kind == TokenKind::LowerIdentifier
        && p.tokens[p.pos + 2].kind == TokenKind::Colon
        && scan_for_close_before_comma(p, open, close)
    {
        let open_tok = p.bump();
        let name_tok = p.bump();
        p.bump();
        let ty = parse_expr(p);
        let close_tok = p.expect(close);
        Some(mk(
            Span::new(open_tok.span.file, open_tok.span.start, close_tok.span.end),
            lexeme_to_string(name_tok.lexeme),
            Box::new(ty),
        ))
    } else {
        None
    }
}

fn scan_for_close_before_comma(p: &Parser, open: TokenKind, close: TokenKind) -> bool {
    let mut depth = 0;
    let mut i = p.pos;
    while i < p.tokens.len() {
        let kind = p.tokens[i].kind;
        if kind == open {
            depth += 1;
        } else if kind == close {
            depth -= 1;
            if depth == 0 {
                return true;
            }
        } else if kind == TokenKind::Comma && depth == 1 {
            return false;
        }
        i += 1;
    }
    false
}

fn parse_pattern(p: &mut Parser) -> SyntaxPattern {
    match p.peek() {
        TokenKind::Underscore => {
            let tok = p.bump();
            SyntaxPattern::Wildcard(tok.span)
        }
        TokenKind::LParen => {
            let lparen = p.bump();
            let mut pats = Vec::new();
            if !p.at(TokenKind::RParen) {
                pats.push(parse_pattern(p));
                while p.at(TokenKind::Comma) {
                    p.bump();
                    pats.push(parse_pattern(p));
                }
            }
            let rparen = p.expect(TokenKind::RParen);
            if pats.len() == 1 {
                pats.into_iter().next().unwrap()
            } else {
                SyntaxPattern::Tuple {
                    elements: pats,
                    span: Span::new(lparen.span.file, lparen.span.start, rparen.span.end),
                }
            }
        }
        TokenKind::UpperIdentifier | TokenKind::LowerIdentifier => {
            let qn = parse_qualified_name(p);
            if qn.is_upper || !qn.namespace.is_empty() {
                let mut args = Vec::new();
                while is_pattern_atom_start(p.peek()) {
                    args.push(parse_pattern_atom(p));
                }
                let span = if args.is_empty() {
                    qn.span
                } else {
                    use crate::syntax::Spanned;
                    Span::new(
                        qn.span.file,
                        qn.span.start,
                        args.last().unwrap().span().end,
                    )
                };
                SyntaxPattern::Constructor {
                    namespace: qn.namespace,
                    name: qn.name,
                    args,
                    span,
                }
            } else {
                SyntaxPattern::Var(qn.name, qn.span)
            }
        }
        _ => {
            let span = p.current_span();
            p.errors.push(ParseError {
                kind: ParseErrorKind::UnexpectedToken,
                span,
                expected: Vec::new(),
                found: Some(p.peek()),
            });
            SyntaxPattern::Wildcard(span)
        }
    }
}

fn is_pattern_atom_start(kind: TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Underscore
            | TokenKind::LParen
            | TokenKind::UpperIdentifier
            | TokenKind::LowerIdentifier
    )
}

fn parse_pattern_atom(p: &mut Parser) -> SyntaxPattern {
    match p.peek() {
        TokenKind::Underscore => {
            let tok = p.bump();
            SyntaxPattern::Wildcard(tok.span)
        }
        TokenKind::LParen => {
            let lparen = p.bump();
            let mut pats = Vec::new();
            if !p.at(TokenKind::RParen) {
                pats.push(parse_pattern(p));
                while p.at(TokenKind::Comma) {
                    p.bump();
                    pats.push(parse_pattern(p));
                }
            }
            let rparen = p.expect(TokenKind::RParen);
            if pats.len() == 1 {
                pats.into_iter().next().unwrap()
            } else {
                SyntaxPattern::Tuple {
                    elements: pats,
                    span: Span::new(lparen.span.file, lparen.span.start, rparen.span.end),
                }
            }
        }
        TokenKind::UpperIdentifier | TokenKind::LowerIdentifier => {
            let qn = parse_qualified_name(p);
            if qn.is_upper || !qn.namespace.is_empty() {
                SyntaxPattern::Constructor {
                    namespace: qn.namespace,
                    name: qn.name,
                    args: Vec::new(),
                    span: qn.span,
                }
            } else {
                SyntaxPattern::Var(qn.name, qn.span)
            }
        }
        _ => {
            let span = p.current_span();
            p.errors.push(ParseError {
                kind: ParseErrorKind::UnexpectedToken,
                span,
                expected: Vec::new(),
                found: Some(p.peek()),
            });
            SyntaxPattern::Wildcard(span)
        }
    }
}

fn parse_expr(p: &mut Parser) -> Expr {
    match p.peek() {
        TokenKind::Lambda => return parse_lambda(p),
        TokenKind::Let => return parse_let(p),
        _ => {}
    }

    if let Some(binder) = try_parse_binder(p) {
        if p.at(TokenKind::Arrow) {
            p.bump();
            let body = parse_expr(p);
            use crate::syntax::Spanned;
            return Expr::Pi {
                span: Span::new(binder.span().file, binder.span().start, body.span().end),
                binder,
                codomain: Box::new(body),
            };
        } else if p.at(TokenKind::Product) {
            p.bump();
            let body = parse_expr(p);
            use crate::syntax::Spanned;
            return Expr::Sigma {
                span: Span::new(binder.span().file, binder.span().start, body.span().end),
                binder,
                codomain: Box::new(body),
            };
        }
        let expr = match &binder {
            SyntaxBinder::Explicit(_, _, ty)
            | SyntaxBinder::Implicit(_, _, ty)
            | SyntaxBinder::Instance(_, _, ty) => (**ty).clone(),
        };
        return parse_arrow_or_product_rhs(p, expr);
    }

    let lhs = parse_cmp(p);
    parse_arrow_or_product_rhs(p, lhs)
}

fn parse_arrow_or_product_rhs(p: &mut Parser, lhs: Expr) -> Expr {
    use crate::syntax::Spanned;
    if p.at(TokenKind::Arrow) {
        p.bump();
        let rhs = parse_expr(p);
        Expr::Arrow {
            span: Span::new(lhs.span().file, lhs.span().start, rhs.span().end),
            param_type: Box::new(lhs),
            return_type: Box::new(rhs),
        }
    } else if p.at(TokenKind::Product) {
        p.bump();
        let rhs = parse_expr(p);
        let lhs_span = lhs.span();
        Expr::Sigma {
            span: Span::new(lhs_span.file, lhs_span.start, rhs.span().end),
            binder: SyntaxBinder::Explicit(lhs_span, String::from("_"), Box::new(lhs)),
            codomain: Box::new(rhs),
        }
    } else {
        lhs
    }
}

fn parse_lambda(p: &mut Parser) -> Expr {
    let kw = p.bump(); // λ or \
    let mut binders = Vec::new();
    while let Some(b) = try_parse_binder(p) {
        binders.push(b);
    }
    if binders.is_empty() {
        let span = p.current_span();
        p.errors.push(ParseError {
            kind: ParseErrorKind::UnexpectedToken,
            span,
            expected: alloc::vec![TokenKind::LParen, TokenKind::LBrace, TokenKind::LBracket],
            found: Some(p.peek()),
        });
    }
    p.expect(TokenKind::FatArrow);
    let body = parse_expr(p);
    use crate::syntax::Spanned;
    Expr::Lambda {
        span: Span::new(kw.span.file, kw.span.start, body.span().end),
        binders,
        body: Box::new(body),
    }
}

fn parse_let(p: &mut Parser) -> Expr {
    let kw = p.bump();
    let name_tok = p.expect(TokenKind::LowerIdentifier);

    let type_ann = if p.at(TokenKind::Colon) {
        p.bump();
        let ty = parse_expr(p);
        Some(Box::new(ty))
    } else {
        None
    };

    p.expect(TokenKind::Equal);
    let value = parse_expr(p);
    p.expect(TokenKind::In);
    let body = parse_expr(p);
    use crate::syntax::Spanned;
    Expr::Let {
        span: Span::new(kw.span.file, kw.span.start, body.span().end),
        name: lexeme_to_string(name_tok.lexeme),
        type_ann,
        value: Box::new(value),
        body: Box::new(body),
    }
}

fn parse_cmp(p: &mut Parser) -> Expr {
    let lhs = parse_add(p);
    let op = match p.peek() {
        TokenKind::EqualEqual => InfixOp::Eq,
        TokenKind::BangEqual => InfixOp::Neq,
        TokenKind::LessEqual => InfixOp::Leq,
        TokenKind::GreaterEqual => InfixOp::Geq,
        TokenKind::Less => InfixOp::Lt,
        TokenKind::Greater => InfixOp::Gt,
        _ => return lhs,
    };
    p.bump();
    let rhs = parse_add(p);
    use crate::syntax::Spanned;
    Expr::InfixOp {
        span: Span::new(lhs.span().file, lhs.span().start, rhs.span().end),
        op,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
    }
}

fn parse_add(p: &mut Parser) -> Expr {
    let mut lhs = parse_mul(p);
    loop {
        let op = match p.peek() {
            TokenKind::Plus => InfixOp::Add,
            TokenKind::Minus => InfixOp::Sub,
            _ => break,
        };
        p.bump();
        let rhs = parse_mul(p);
        use crate::syntax::Spanned;
        lhs = Expr::InfixOp {
            span: Span::new(lhs.span().file, lhs.span().start, rhs.span().end),
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        };
    }
    lhs
}

fn parse_mul(p: &mut Parser) -> Expr {
    let mut lhs = parse_app(p);
    loop {
        let op = match p.peek() {
            TokenKind::Star => InfixOp::Mul,
            TokenKind::Slash => InfixOp::Div,
            _ => break,
        };
        p.bump();
        let rhs = parse_app(p);
        use crate::syntax::Spanned;
        lhs = Expr::InfixOp {
            span: Span::new(lhs.span().file, lhs.span().start, rhs.span().end),
            op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        };
    }
    lhs
}

fn parse_app(p: &mut Parser) -> Expr {
    let mut lhs = parse_proj(p);
    while is_atom_start(p.peek()) {
        let rhs = parse_proj(p);
        use crate::syntax::Spanned;
        lhs = Expr::App {
            span: Span::new(lhs.span().file, lhs.span().start, rhs.span().end),
            fun: Box::new(lhs),
            arg: Box::new(rhs),
        };
    }
    lhs
}

fn parse_proj(p: &mut Parser) -> Expr {
    let mut lhs = parse_expr_atom(p);
    while p.at(TokenKind::Dot) {
        p.bump();
        let field_tok = p.expect(TokenKind::LowerIdentifier);
        use crate::syntax::Spanned;
        lhs = Expr::Proj {
            span: Span::new(lhs.span().file, lhs.span().start, field_tok.span.end),
            value: Box::new(lhs),
            field: lexeme_to_string(field_tok.lexeme),
        };
    }
    lhs
}

fn is_atom_start(kind: TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::LowerIdentifier
            | TokenKind::UpperIdentifier
            | TokenKind::Number
            | TokenKind::String
            | TokenKind::Underscore
            | TokenKind::LParen
            | TokenKind::LBracket
    )
}

fn parse_expr_atom(p: &mut Parser) -> Expr {
    match p.peek() {
        TokenKind::Number => {
            let tok = p.bump();
            let s = lexeme_to_string(tok.lexeme);
            let n = s.parse::<u64>().unwrap_or(0);
            Expr::Lit {
                value: Literal::Nat(n),
                span: tok.span,
            }
        }
        TokenKind::String => {
            let tok = p.bump();
            let s = lexeme_to_string(tok.lexeme);
            let inner = if s.len() >= 2 {
                &s[1..s.len() - 1]
            } else {
                &s
            };
            Expr::Lit {
                value: Literal::Str(String::from(inner)),
                span: tok.span,
            }
        }
        TokenKind::Underscore => {
            let tok = p.bump();
            Expr::Hole(tok.span)
        }
        TokenKind::LParen => {
            let lparen = p.bump();
            let mut items = Vec::new();
            if !p.at(TokenKind::RParen) {
                items.push(parse_expr(p));
                while p.at(TokenKind::Comma) {
                    p.bump();
                    items.push(parse_expr(p));
                }
            }
            let rparen = p.expect(TokenKind::RParen);
            let span = Span::new(lparen.span.file, lparen.span.start, rparen.span.end);
            match items.len() {
                0 => Expr::Unit(span),
                1 => items.into_iter().next().unwrap(),
                _ => Expr::Tuple {
                    elements: items,
                    span,
                },
            }
        }
        TokenKind::LBracket => {
            let lbracket = p.bump();
            let mut items = Vec::new();
            if !p.at(TokenKind::RBracket) {
                items.push(parse_expr(p));
                while p.at(TokenKind::Comma) {
                    p.bump();
                    items.push(parse_expr(p));
                }
            }
            let rbracket = p.expect(TokenKind::RBracket);
            Expr::Array {
                elements: items,
                span: Span::new(lbracket.span.file, lbracket.span.start, rbracket.span.end),
            }
        }
        TokenKind::LowerIdentifier | TokenKind::UpperIdentifier => {
            let qn = parse_qualified_name(p);
            if qn.is_upper {
                Expr::Constructor {
                    name: qn.name,
                    namespace: qn.namespace,
                    span: qn.span,
                }
            } else {
                Expr::Var {
                    namespace: qn.namespace,
                    member: qn.name,
                    span: qn.span,
                }
            }
        }
        _ => {
            let span = p.current_span();
            let found = if p.pos < p.tokens.len() {
                Some(p.tokens[p.pos].kind)
            } else {
                None
            };
            p.errors.push(ParseError {
                kind: if found.is_some() {
                    ParseErrorKind::UnexpectedToken
                } else {
                    ParseErrorKind::UnexpectedEndOfInput
                },
                span,
                expected: Vec::new(),
                found,
            });
            // Advance to avoid infinite loop
            if p.pos < p.tokens.len() {
                p.pos += 1;
            }
            Expr::Hole(span)
        }
    }
}

// ── Qualified names ─────────────────────────────────────────────────────

struct QualifiedName {
    namespace: Vec<String>,
    name: String,
    is_upper: bool,
    span: Span,
}

fn parse_qualified_name(p: &mut Parser) -> QualifiedName {
    let first = p.bump();
    let mut parts = alloc::vec![(lexeme_to_string(first.lexeme), first.kind == TokenKind::UpperIdentifier, first.span)];

    while p.at(TokenKind::DoubleColon) {
        p.bump();
        if p.at(TokenKind::LowerIdentifier) || p.at(TokenKind::UpperIdentifier) {
            let tok = p.bump();
            parts.push((
                lexeme_to_string(tok.lexeme),
                tok.kind == TokenKind::UpperIdentifier,
                tok.span,
            ));
        } else {
            break;
        }
    }

    let last = parts.last().unwrap();
    let span = Span::new(first.span.file, first.span.start, last.2.end);

    if parts.len() == 1 {
        QualifiedName {
            namespace: Vec::new(),
            name: parts.remove(0).0,
            is_upper: first.kind == TokenKind::UpperIdentifier,
            span,
        }
    } else {
        let is_upper = parts.last().unwrap().1;
        let name = parts.last().unwrap().0.clone();
        let namespace = parts[..parts.len() - 1]
            .iter()
            .map(|(n, _, _)| n.clone())
            .collect();
        QualifiedName {
            namespace,
            name,
            is_upper,
            span,
        }
    }
}
