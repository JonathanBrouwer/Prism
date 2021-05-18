use std::ops::Range;

use logos::{Lexer, Logos, Source};

use crate::lexer::logos_lexer::LogosLexerToken;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LexerToken<'a> {
    pub span: Range<usize>,
    pub token: LexerTokenType<'a>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LexerTokenType<'a> {
    ParenOpen,
    ParenClose,
    BracketOpen,
    BracketClose,
    Identifier(&'a str),
    Line,
    BlockStart,
    BlockStop,
    Error,
}

pub struct JonlaLexer<'a> {
    logos_lexer: Lexer<'a, LogosLexerToken>,

    // Indent tracker
    after_line: bool,
    indent: usize,
    inserted_final_newline: bool,
}

impl<'a> JonlaLexer<'a> {
    pub fn lexer(inp: &'a str) -> JonlaLexer<'a> {
        JonlaLexer {
            logos_lexer: LogosLexerToken::lexer(inp),
            after_line: true,
            indent: 0,
            inserted_final_newline: false,
        }
    }
}

impl<'a> Iterator for JonlaLexer<'a> {
    type Item = LexerToken<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let start = self.logos_lexer.span().end;

        if self.after_line {
            //Get new indent
            let rem: &str = self.logos_lexer.remainder();
            let space_count = rem.chars().take_while(|&c| c == ' ').count();

            //Check if indent is valid, calculate new indent
            if space_count % 4 != 0 {
                //TODO make a best guess on what the new indent level is?
                self.after_line = false;
                return Some(LexerToken {
                    token: LexerTokenType::Error,
                    span: start..start + space_count,
                });
            }
            let indent_new = space_count / 4;

            //Do something based on indent change
            if indent_new == self.indent {
                self.after_line = false;
            } else if indent_new > self.indent {
                self.indent += 1;
                return Some(LexerToken {
                    token: LexerTokenType::BlockStart,
                    span: start..start + space_count,
                });
            } else if indent_new < self.indent {
                self.indent -= 1;
                return Some(LexerToken {
                    token: LexerTokenType::BlockStop,
                    span: start..start + space_count,
                });
            }
        }

        //Find next token
        let next = self.logos_lexer.next();

        //If this is a line token,
        if let Some(LogosLexerToken::Line) = next {
            self.after_line = true;
        }
        //If this is the end of the file
        else if let None = next {
            if !self.inserted_final_newline {
                self.inserted_final_newline = true;
                return Some(LexerToken {
                    token: LexerTokenType::Line,
                    span: start..start,
                });
            }
            if self.indent > 0 {
                self.indent -= 1;
                return Some(LexerToken {
                    token: LexerTokenType::BlockStop,
                    span: start..start,
                });
            }
        }

        next.map(|l| match l {
            LogosLexerToken::ParenOpen => LexerToken {
                token: LexerTokenType::ParenOpen,
                span: self.logos_lexer.span(),
            },
            LogosLexerToken::ParenClose => LexerToken {
                token: LexerTokenType::ParenClose,
                span: self.logos_lexer.span(),
            },
            LogosLexerToken::BracketOpen => LexerToken {
                token: LexerTokenType::BracketOpen,
                span: self.logos_lexer.span(),
            },
            LogosLexerToken::BracketClose => LexerToken {
                token: LexerTokenType::BracketClose,
                span: self.logos_lexer.span(),
            },
            LogosLexerToken::Identifier => LexerToken {
                token: LexerTokenType::Identifier(
                    self.logos_lexer
                        .source()
                        .slice(self.logos_lexer.span())
                        .unwrap(),
                ),
                span: self.logos_lexer.span(),
            },
            LogosLexerToken::Line => LexerToken {
                token: LexerTokenType::Line,
                span: self.logos_lexer.span(),
            },
            LogosLexerToken::Error => LexerToken {
                token: LexerTokenType::Error,
                span: self.logos_lexer.span(),
            },
        })
    }
}
