use crate::syntax::{
    SourceFile, SourcePos, SourceSpan,
    error::{LexError, LexErrorKind},
    token::{Token, TokenKind},
};

fn decode_utf8_char(bytes: &[u8]) -> Option<char> {
    if bytes.is_empty() {
        return None;
    }
    let s = core::str::from_utf8(bytes).ok()?;
    s.chars().next()
}

fn is_ident_start(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

fn is_ident_continue(c: char) -> bool {
    c.is_alphabetic() || c.is_ascii_digit() || c == '_'
}

pub struct Lexer<'a> {
    source_file: &'a SourceFile<'a>,
    source_pos: SourcePos,
}

impl<'a> Lexer<'a> {
    pub fn new(source_file: &'a SourceFile<'a>) -> Self {
        Self {
            source_file,
            source_pos: SourcePos::start_of_file(source_file.id),
        }
    }

    pub fn read_all(&mut self, tokens: &mut [Token<'a>]) -> usize {
        let mut index = 0;
        while let Some(Ok(token)) = self.next() {
            if index < tokens.len() {
                tokens[index] = token;
                index += 1;
            } else {
                break;
            }
        }
        index
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Token<'a>, LexError<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        let source = &self.source_file.source;

        loop {
            if self.source_pos.byte_offset >= source.len() {
                return None;
            }
            let b = source[self.source_pos.byte_offset];
            match b {
                b' ' | b'\t' | b'\r' => self.source_pos.row(),
                b'\n' => self.source_pos.newline(),
                _ => break,
            }
        }

        let remaining = &source[self.source_pos.byte_offset..];
        let current = decode_utf8_char(remaining)?;
        let start = self.source_pos;

        match current {
            '0'..='9' => {
                while self.source_pos.byte_offset < source.len() {
                    let c = source[self.source_pos.byte_offset] as char;
                    if c.is_ascii_digit() {
                        self.source_pos.row();
                    } else {
                        break;
                    }
                }
                let lexeme = &source[start.byte_offset..self.source_pos.byte_offset];

                Some(Ok(Token {
                    kind: TokenKind::Number,
                    lexeme,
                    span: SourceSpan {
                        start,
                        end: self.source_pos,
                    },
                }))
            }
            c if is_ident_start(c) => {
                while self.source_pos.byte_offset < source.len() {
                    let remaining = &source[self.source_pos.byte_offset..];
                    if let Some(c) = decode_utf8_char(remaining) {
                        if is_ident_continue(c) {
                            self.source_pos.advance_by_char(c);
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                let lexeme = &source[start.byte_offset..self.source_pos.byte_offset];
                let kind = match lexeme {
                    b"struct" => TokenKind::Struct,
                    _ => TokenKind::Identifier,
                };

                Some(Ok(Token {
                    kind,
                    lexeme,
                    span: SourceSpan {
                        start,
                        end: self.source_pos,
                    },
                }))
            }
            '"' => {
                self.source_pos.row();
                while self.source_pos.byte_offset < source.len() {
                    let c = source[self.source_pos.byte_offset] as char;
                    if c == '"' {
                        self.source_pos.row();
                        break;
                    } else {
                        self.source_pos.row();
                    }
                }
                let lexeme = &source[start.byte_offset..self.source_pos.byte_offset];

                Some(Ok(Token {
                    kind: TokenKind::String,
                    lexeme,
                    span: SourceSpan {
                        start,
                        end: self.source_pos,
                    },
                }))
            }
            '=' => {
                self.source_pos.row();
                let lexeme = &source[start.byte_offset..self.source_pos.byte_offset];

                Some(Ok(Token {
                    kind: TokenKind::Equal,
                    lexeme,
                    span: SourceSpan {
                        start,
                        end: self.source_pos,
                    },
                }))
            }
            ',' => {
                self.source_pos.row();
                let lexeme = &source[start.byte_offset..self.source_pos.byte_offset];

                Some(Ok(Token {
                    kind: TokenKind::Comma,
                    lexeme,
                    span: SourceSpan {
                        start,
                        end: self.source_pos,
                    },
                }))
            }
            ':' => {
                self.source_pos.row();
                let lexeme = &source[start.byte_offset..self.source_pos.byte_offset];
                Some(Ok(Token {
                    kind: TokenKind::Colon,
                    lexeme,
                    span: SourceSpan {
                        start,
                        end: self.source_pos,
                    },
                }))
            }
            '{' => {
                self.source_pos.row();
                let lexeme = &source[start.byte_offset..self.source_pos.byte_offset];
                Some(Ok(Token {
                    kind: TokenKind::LBrace,
                    lexeme,
                    span: SourceSpan {
                        start,
                        end: self.source_pos,
                    },
                }))
            }
            '}' => {
                self.source_pos.row();
                let lexeme = &source[start.byte_offset..self.source_pos.byte_offset];
                Some(Ok(Token {
                    kind: TokenKind::RBrace,
                    lexeme,
                    span: SourceSpan {
                        start,
                        end: self.source_pos,
                    },
                }))
            }
            '(' => {
                self.source_pos.row();
                let lexeme = &source[start.byte_offset..self.source_pos.byte_offset];
                Some(Ok(Token {
                    kind: TokenKind::LParen,
                    lexeme,
                    span: SourceSpan {
                        start,
                        end: self.source_pos,
                    },
                }))
            }
            ')' => {
                self.source_pos.row();
                let lexeme = &source[start.byte_offset..self.source_pos.byte_offset];
                Some(Ok(Token {
                    kind: TokenKind::RParen,
                    lexeme,
                    span: SourceSpan {
                        start,
                        end: self.source_pos,
                    },
                }))
            }
            ';' => {
                self.source_pos.row();
                let lexeme = &source[start.byte_offset..self.source_pos.byte_offset];
                Some(Ok(Token {
                    kind: TokenKind::Semicolon,
                    lexeme,
                    span: SourceSpan {
                        start,
                        end: self.source_pos,
                    },
                }))
            }
            u => {
                self.source_pos.advance_by_char(u);
                let lexeme = &source[start.byte_offset..self.source_pos.byte_offset];
                Some(Err(LexError {
                    kind: LexErrorKind::UnexpectedChar(u),
                    span: SourceSpan {
                        start,
                        end: self.source_pos,
                    },
                    lexeme,
                }))
            }
        }
    }
}
