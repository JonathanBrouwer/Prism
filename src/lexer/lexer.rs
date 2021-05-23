use logos::{Lexer, Logos};

use std::ops::Range;
use crate::lexer::logos::LogosToken;
use std::collections::VecDeque;
use std::cmp::Ordering;
use crate::lexer::lexer::LexerToken::*;


#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LexerItem<'a> {
    pub span: Range<usize>,
    pub token: LexerToken<'a>
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LexerToken<'a> {
    Name(&'a str),
    Control(&'a str),
    BlockStart,
    BlockStop,
    Line,
    EOF,
    Error(&'a str)
}

impl<'a> LexerToken<'a> {
    pub fn to_type(&self) -> LexerTokenType {
        match self {
            Name(_) => LexerTokenType::Name,
            Control(_) => LexerTokenType::Control,
            BlockStart => LexerTokenType::BlockStart,
            BlockStop => LexerTokenType::BlockStop,
            Line => LexerTokenType::Line,
            EOF => LexerTokenType::EOF,
            Error(_) => LexerTokenType::Error
        }
    }

    pub fn unwrap_name(&self) -> &'a str {
        match self {
            Name(n) => n,
            _ => panic!("Expected name!")
        }
    }

    pub fn unwrap_control(&self) -> &'a str {
        match self {
            Control(n) => n,
            _ => panic!("Expected control!")
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LexerTokenType {
    Name,
    Control,
    BlockStart,
    BlockStop,
    Line,
    EOF,
    Error
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
                    vec![LexerToken::BlockStop]
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
                            vec![BlockStop, Error("Illegal indentation."), Line]
                        } else {
                            vec![BlockStop, Line]
                        }
                    }
                    Ordering::Greater => {
                        self.blocks.push_back(indent);
                        vec![Line, BlockStart]
                    }
                    Ordering::Equal => {
                        vec![Line]
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