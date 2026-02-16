use crate::parser::ParserPrismEnv;
use prism_diag_derive::Diagnostic;
use prism_input::pos::Pos;
use prism_input::span::Span;
use std::mem;

const SYMBOL_CHARS: &str = "<>,.!@#$%^&*/\\:;|";

#[derive(Copy, Clone)]
pub enum Token {
    Newline(Span),
    Whitespace(Span),
    Comment(Span),
    EOF(Pos),

    OpenParen(Span),
    CloseParen(Span),
    Identifier { span: Span, keyword: bool },
    Symbol(Span),
    StringLit(StrLit),
    NumLit(NumLit),
}

#[derive(Copy, Clone)]
pub struct StrLit {
    pub prefix: Span,
    pub open_quote: Span,
    pub value: Span,
    pub close_quote: Span,
    pub suffix: Span,
}

#[derive(Copy, Clone)]
pub struct NumLit {
    pub value: Span,
    pub suffix: Span,
}

impl Token {
    pub fn span(&self) -> Span {
        match self {
            Token::Whitespace(span)
            | Token::Newline(span)
            | Token::Comment(span)
            | Token::OpenParen(span)
            | Token::CloseParen(span)
            | Token::Symbol(span)
            | Token::Identifier { span, .. } => *span,
            Token::StringLit(StrLit {
                prefix: start,
                suffix: end,
                ..
            })
            | Token::NumLit(NumLit {
                value: start,
                suffix: end,
            }) => start.span_to(*end),
            Token::EOF(pos) => pos.span_to(*pos),
        }
    }
}

pub type Tokens = Vec<Token>;

pub struct LexerState {
    pos: Pos,
    tokens: Tokens,
    paren_stack: Vec<Span>,
}

impl LexerState {
    pub fn new(pos: Pos) -> Self {
        Self {
            pos,
            tokens: Tokens::default(),
            paren_stack: vec![],
        }
    }
}

impl<'a> ParserPrismEnv<'a> {
    fn next_char(&mut self, f: impl Fn(char) -> bool) -> Option<(char, Span)> {
        let (next_char, next_pos) = self.lexer.pos.next(&self.db.input)?;
        if !f(next_char) {
            return None;
        }
        let span = self.lexer.pos.span_to(next_pos);
        self.lexer.pos = next_pos;
        Some((next_char, span))
    }

    pub fn next_token_incl_layout(&mut self) -> Token {
        let mut invalid_token_start = None;
        let token = loop {
            let Some((ch, ch_span)) = self.next_char(|_| true) else {
                break Token::EOF(self.lexer.pos);
            };
            break match ch {
                // Newline
                '\r' => {
                    let span = if let Some(('\n', nl_span)) = self.next_char(|c| c == '\n') {
                        ch_span.span_to(nl_span)
                    } else {
                        self.db
                            .push_error(CarriageReturnWithoutNewline { span: ch_span });
                        ch_span
                    };
                    Token::Newline(span)
                }
                '\n' => Token::Newline(ch_span),
                // Whitespace
                ch if ch.is_whitespace() => {
                    while self.next_char(|c| c.is_whitespace()).is_some() {}
                    Token::Whitespace(ch_span.span_to_pos(self.lexer.pos))
                }
                // Comment
                '/' => {
                    if let Some(_) = self.next_char(|c| c == '/') {
                        while let Some(_) = self.next_char(|_| true) {
                            if ch == '\n' {
                                break;
                            }
                        }
                        Token::Comment(ch_span.span_to_pos(self.lexer.pos))
                    } else if let Some(_) = self.next_char(|c| c == '*') {
                        //TODO incomplete block comment
                        while let Some(_) = self.next_char(|_| true) {
                            if ch == '*' && self.next_char(|c| c == '/').is_some() {
                                break;
                            }
                        }
                        Token::Comment(ch_span.span_to_pos(self.lexer.pos))
                    } else {
                        Token::Symbol(ch_span)
                    }
                }
                // OpenParen
                '(' | '{' | '[' => {
                    self.lexer.paren_stack.push(ch_span);
                    Token::OpenParen(ch_span)
                }
                // CloseParen
                ')' | '}' | ']' => {
                    let expected_open_char = match ch {
                        ')' => '(',
                        '}' => '{',
                        ']' => '[',
                        _ => unreachable!(),
                    };
                    let Some(last_open_span) = self.lexer.paren_stack.last().copied() else {
                        self.db.push_error(MismatchedClosingDelim {
                            span: ch_span,
                            last_open_span: None,
                        });
                        continue;
                    };
                    let last_open_char = {
                        let input = self.db.input.inner();
                        let last_open_str = input.slice(last_open_span);
                        assert_eq!(last_open_str.len(), 1);
                        last_open_str.chars().next().unwrap()
                    };
                    if last_open_char != expected_open_char {
                        self.db.push_error(MismatchedClosingDelim {
                            span: ch_span,
                            last_open_span: Some(last_open_span),
                        });
                    };
                    self.lexer.paren_stack.pop();
                    Token::CloseParen(ch_span)
                }
                c if unicode_ident::is_xid_start(c) => {
                    while let Some(_) = self.next_char(|c| unicode_ident::is_xid_continue(c)) {}
                    Token::Identifier {
                        span: ch_span.span_to_pos(self.lexer.pos),
                        // `keyword` will be set to true by the parser if applicable
                        keyword: false,
                    }
                }
                c if SYMBOL_CHARS.contains(c) => Token::Symbol(ch_span),
                _ => {
                    if invalid_token_start.is_none() {
                        invalid_token_start = Some(ch_span.start_pos());
                    }
                    continue;
                }
            };
        };
        if let Some(invalid_token_start) = invalid_token_start {
            let invalid_token_span = invalid_token_start.span_to(token.span().start_pos());
            self.db.push_error(InvalidToken {
                span: invalid_token_span,
            });
        }

        self.lexer.tokens.push(token);
        token
    }

    pub fn next_token(&mut self) -> Token {
        loop {
            let token = self.next_token_incl_layout();
            if let Token::Whitespace(..) | Token::Comment(..) = token {
                continue;
            }
            return token;
        }
    }

    pub fn finish_lexing(&mut self) -> Tokens {
        while let Some(open_paren) = self.lexer.paren_stack.pop() {
            self.db.push_error(UnmatchedOpenDelim { span: open_paren });
        }

        let tokens = mem::take(&mut self.lexer.tokens);
        tokens
    }

    pub fn token(&self) -> Token {
        self.lexer
            .tokens
            .last()
            .copied()
            .unwrap_or(Token::EOF(self.lexer.pos))
    }

    pub fn mark_token_keyword(&mut self) {
        let Some(Token::Identifier { keyword, .. }) = self.lexer.tokens.last_mut() else {
            unreachable!()
        };
        *keyword = true;
    }
}

#[derive(Diagnostic)]
#[diag(title = "found a carriage return without a following newline")]
struct CarriageReturnWithoutNewline {
    #[sugg]
    span: Span,
}

#[derive(Diagnostic)]
#[diag(title = "invalid token found")]
struct InvalidToken {
    #[sugg]
    span: Span,
}

#[derive(Diagnostic)]
#[diag(title = "mismatched closing delimiter")]
struct MismatchedClosingDelim {
    #[sugg(label = "mismatched delimiter")]
    span: Span,
    #[allow(unused)]
    last_open_span: Option<Span>,
}

#[derive(Diagnostic)]
#[diag(title = "unmatched open delimiter")]
struct UnmatchedOpenDelim {
    #[sugg(label = "unmatched delimiter")]
    span: Span,
}
