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
    Error(&'a str)
}

pub struct ActualLexer<'a> {
    pub logos: Lexer<'a, LogosToken>,

    // Keep track of block information
    pub blocks: VecDeque<usize>,
    pub queue: VecDeque<LexerItem<'a>>,
}

impl<'a> ActualLexer<'a> {
    pub fn new(source: &'a str) -> ActualLexer<'a> {
        ActualLexer {
            logos: LogosToken::lexer(source),
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