use logos::{Lexer, Logos};

use std::ops::Range;
use crate::lexer::logos::LogosToken;
use std::collections::VecDeque;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use crate::peg_parser::parser_token::*;


#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct LexerToken<'a> {
    pub span: Range<usize>,
    pub token: LexerTokenValue<'a>
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum LexerTokenValue<'a> {
    Name(&'a str),
    Control(&'a str),
    BlockStart,
    BlockEnd,
    Line,
    EOF,
    Error(&'a str)
}

impl<'a> TokenValue for LexerTokenValue<'a> {

}

impl<'a> Token<LexerTokenType, LexerTokenValue<'a>> for LexerToken<'a> {
    fn to_val(&self) -> LexerTokenValue<'a> {
        self.token
    }

    fn to_type(&self) -> LexerTokenType {
        match self.token {
            LexerTokenValue::Name(_) => LexerTokenType::Name,
            LexerTokenValue::Control(_) => LexerTokenType::Control,
            LexerTokenValue::BlockStart => LexerTokenType::BlockStart,
            LexerTokenValue::BlockEnd => LexerTokenType::BlockEnd,
            LexerTokenValue::Line => LexerTokenType::Line,
            LexerTokenValue::EOF => LexerTokenType::EOF,
            LexerTokenValue::Error(_) => LexerTokenType::Error
        }
    }
}

impl<'a> Display for LexerTokenValue<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LexerTokenValue::Name(v) => write!(f, "( name: {} )", v),
            LexerTokenValue::Control(v) => write!(f, "( control: {} )", v),
            LexerTokenValue::BlockStart => write!(f, "start of a block"),
            LexerTokenValue::BlockEnd => write!(f, "end of a block"),
            LexerTokenValue::Line => write!(f, "new line"),
            LexerTokenValue::EOF => write!(f, "end of file"),
            LexerTokenValue::Error(_) => write!(f, "lexer error")
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum LexerTokenType {
    Name,
    Control,
    BlockStart,
    BlockEnd,
    Line,
    EOF,
    Error
}

impl TokenType for LexerTokenType {
}

impl Display for LexerTokenType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LexerTokenType::Name=> write!(f, "name"),
            LexerTokenType::Control => write!(f, "control"),
            LexerTokenType::BlockStart => write!(f, "start of a block"),
            LexerTokenType::BlockEnd => write!(f, "end of a block"),
            LexerTokenType::Line => write!(f, "new line"),
            LexerTokenType::EOF => write!(f, "end of file"),
            LexerTokenType::Error => write!(f, "lexer error")
        }
    }
}

impl<'a> LexerTokenValue<'a> {
    pub fn unwrap_name(&self) -> &'a str {
        match self {
            LexerTokenValue::Name(n) => n,
            _ => panic!("Expected name!")
        }
    }

    pub fn unwrap_control(&self) -> &'a str {
        match self {
            LexerTokenValue::Control(n) => n,
            _ => panic!("Expected control!")
        }
    }
}

pub struct ActualLexer<'a> {
    pub logos: Lexer<'a, LogosToken>,

    // Keep track of EOF information
    pub eof: bool,

    // Keep track of block information
    pub blocks: VecDeque<usize>,
    pub queue: VecDeque<LexerToken<'a>>,
}

impl<'a> ActualLexer<'a> {
    pub fn new(source: &'a str) -> ActualLexer<'a> {
        ActualLexer {
            logos: LogosToken::lexer(source),
            eof: false,
            blocks: VecDeque::from(vec![0]),
            queue: VecDeque::new()
        }
    }
}

impl<'a> Iterator for ActualLexer<'a> {
    type Item = LexerToken<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.queue.len() > 0 {
            self.queue.pop_front()
        } else {
            self.next_multiple().into_iter().for_each(|i| self.queue.push_back(i));
            self.queue.pop_front()
        }
    }
}

impl<'a> ActualLexer<'a> {
    fn next_multiple(&mut self) -> Vec<LexerToken<'a>> {
        match self.logos.next() {
            None => {
                if self.blocks.len() > 1 {
                    self.blocks.pop_back();
                    vec![LexerTokenValue::BlockEnd]
                } else if !self.eof {
                    self.eof = true;
                    vec![LexerTokenValue::EOF]
                } else {
                    vec![]
                }
            },
            Some(LogosToken::Line) => {
                let indent: usize = self.logos.remainder().chars().take_while(|&c| c == ' ').count();
                match indent.cmp(self.blocks.back().unwrap()) {
                    Ordering::Less => {
                        self.blocks.pop_back();
                        // We went too far, this is not a legal structure.
                        if indent > *self.blocks.back().unwrap() {
                            vec![LexerTokenValue::BlockEnd, LexerTokenValue::Error("Illegal indentation."), LexerTokenValue::Line]
                        } else {
                            vec![LexerTokenValue::BlockEnd, LexerTokenValue::Line]
                        }
                    }
                    Ordering::Greater => {
                        self.blocks.push_back(indent);
                        vec![LexerTokenValue::Line, LexerTokenValue::BlockStart]
                    }
                    Ordering::Equal => {
                        vec![LexerTokenValue::Line]
                    }
                }

            },
            Some(LogosToken::Name) => {
                vec![LexerTokenValue::Name(self.logos.slice())]
            },
            Some(LogosToken::Control) => {
                vec![LexerTokenValue::Control(self.logos.slice())]
            },
            Some(LogosToken::Error) => {
                vec![LexerTokenValue::Error("Illegal character.")]
            }
        }.into_iter().map(|t| LexerToken { span: self.logos.span(), token: t }).collect()
    }
}