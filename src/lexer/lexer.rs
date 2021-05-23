use logos::{Lexer, Logos};

use std::ops::Range;
use crate::lexer::logos::LogosToken;
use std::collections::VecDeque;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};


#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LexerItem<'a> {
    pub span: Range<usize>,
    pub token: LexerToken<'a>
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LexerToken<'a> {
    Name(&'a str),
    Control(&'a str),
    BlockStart,
    BlockEnd,
    Line,
    EOF,
    Error(&'a str)
}

impl<'a> Display for LexerToken<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LexerToken::Name(v) => write!(f, "( name: {} )", v),
            LexerToken::Control(v) => write!(f, "( control: {} )", v),
            LexerToken::BlockStart => write!(f, "start of a block"),
            LexerToken::BlockEnd => write!(f, "end of a block"),
            LexerToken::Line => write!(f, "new line"),
            LexerToken::EOF => write!(f, "end of file"),
            LexerToken::Error(_) => write!(f, "lexer error")
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LexerTokenType {
    Name,
    Control,
    BlockStart,
    BlockEnd,
    Line,
    EOF,
    Error
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

impl<'a> LexerToken<'a> {
    pub fn to_type(&self) -> LexerTokenType {
        match self {
            LexerToken::Name(_) => LexerTokenType::Name,
            LexerToken::Control(_) => LexerTokenType::Control,
            LexerToken::BlockStart => LexerTokenType::BlockStart,
            LexerToken::BlockEnd => LexerTokenType::BlockEnd,
            LexerToken::Line => LexerTokenType::Line,
            LexerToken::EOF => LexerTokenType::EOF,
            LexerToken::Error(_) => LexerTokenType::Error
        }
    }

    pub fn unwrap_name(&self) -> &'a str {
        match self {
            LexerToken::Name(n) => n,
            _ => panic!("Expected name!")
        }
    }

    pub fn unwrap_control(&self) -> &'a str {
        match self {
            LexerToken::Control(n) => n,
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
    pub queue: VecDeque<LexerItem<'a>>,
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
    type Item = LexerItem<'a>;

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
    fn next_multiple(&mut self) -> Vec<LexerItem<'a>> {
        match self.logos.next() {
            None => {
                if self.blocks.len() > 1 {
                    self.blocks.pop_back();
                    vec![LexerToken::BlockEnd]
                } else if !self.eof {
                    self.eof = true;
                    vec![LexerToken::EOF]
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
                            vec![LexerToken::BlockEnd, LexerToken::Error("Illegal indentation."), LexerToken::Line]
                        } else {
                            vec![LexerToken::BlockEnd, LexerToken::Line]
                        }
                    }
                    Ordering::Greater => {
                        self.blocks.push_back(indent);
                        vec![LexerToken::Line, LexerToken::BlockStart]
                    }
                    Ordering::Equal => {
                        vec![LexerToken::Line]
                    }
                }

            },
            Some(LogosToken::Name) => {
                vec![LexerToken::Name(self.logos.slice())]
            },
            Some(LogosToken::Control) => {
                vec![LexerToken::Control(self.logos.slice())]
            },
            Some(LogosToken::Error) => {
                vec![LexerToken::Error("Illegal character.")]
            }
        }.into_iter().map(|t| LexerItem { span: self.logos.span(), token: t }).collect()
    }
}