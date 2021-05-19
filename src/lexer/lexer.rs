use logos::{Lexer, Logos};

use std::ops::Range;
use crate::lexer::logos::LogosToken;
use std::collections::VecDeque;
use std::cmp::Ordering;


#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LexerItem<'a> {
    pub span: Range<usize>,
    pub token: LexerToken<'a>
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LexerToken<'a> {
    Name(&'a str),
    BlockStart,
    BlockStop,
    Line,
    Error(&'a str)
}

pub struct ActualLexer<'a> {
    pub logos: Lexer<'a, LogosToken>,

    // Keep track of block information
    pub new_line: bool,
    pub should_error: bool,
    pub blocks: VecDeque<usize>,
}

impl<'a> ActualLexer<'a> {
    pub fn new(source: &'a str) -> ActualLexer<'a> {
        ActualLexer {
            logos: LogosToken::lexer(source),
            new_line: false,
            should_error: false,
            blocks: VecDeque::from(vec![0])
        }
    }
}

impl<'a> Iterator for ActualLexer<'a> {
    type Item = LexerItem<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.should_error {
            self.should_error = false;
            return Some(LexerItem { span: self.logos.span(), token: LexerToken::Error("Illegal indentation.") })
        }
        if self.new_line {
            let indent: usize = self.logos.remainder().chars().take_while(|&c| c == ' ').count();
            match indent.cmp(self.blocks.back().unwrap()) {
                Ordering::Less => {
                    self.blocks.pop_back();
                    // We went too far, this is not a legal structure.
                    if indent > *self.blocks.back().unwrap() {
                        self.should_error = true;
                    }
                    return Some(LexerItem { span: self.logos.span(), token: LexerToken::BlockStop })
                }
                Ordering::Greater => {
                    self.blocks.push_back(indent);
                    self.new_line = false;
                    return Some(LexerItem { span: self.logos.span(), token: LexerToken::BlockStart })
                }
                Ordering::Equal => {}
            }
        }

        match self.logos.next() {
            None => {
                if self.blocks.len() > 1 {
                    self.blocks.pop_back();
                    Some(LexerToken::BlockStop)
                } else {
                    None
                }
            },
            Some(LogosToken::Line) => {
                self.new_line = true;
                Some(LexerToken::Line)
            },
            Some(LogosToken::Name) => {
                Some(LexerToken::Name(self.logos.slice()))
            },
            Some(LogosToken::Error) => {
                Some(LexerToken::Error("Illegal character."))
            }
        }.map(|t| LexerItem { span: self.logos.span(), token: t })
    }
}