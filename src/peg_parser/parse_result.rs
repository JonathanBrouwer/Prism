use crate::peg_parser::peg_parser::*;
use std::cmp::Ordering;

#[derive(Eq, PartialEq, Clone)]
pub struct ParseSuccess<'a, TT: TokenType, T: Token<TT>> {
    pub(crate) result: (),
    pub(crate) best_error: Option<ParseError<'a, TT, T>>,
    pub(crate) rest: &'a [T],
}

#[derive(Eq, PartialEq, Clone)]
pub enum Expected<TT: TokenType, T: Token<TT>> {
    LiteralExact(T),
    LiteralBind(TT),
}

#[derive(Eq, PartialEq, Clone)]
pub struct ParseError<'a, TT: TokenType, T: Token<TT>> {
    pub on: &'a T,
    pub expect: Vec<Expected<TT, T>>,
    pub inv_priority: usize
}

impl<'a, TT: TokenType, T: Token<TT>> ParseError<'a, TT, T> {
    pub fn combine(mut self, other: ParseError<'a, TT, T>) -> ParseError<'a, TT, T> {
        match self.inv_priority.cmp(&other.inv_priority) {
            // One has higher priority
            Ordering::Less => self,
            Ordering::Greater => other,
            // Equal priority
            Ordering::Equal => {
                for ex in other.expect {
                    if !self.expect.contains(&ex) {
                        self.expect.push(ex)
                    }
                }
                self
            }
        }
    }
}

pub fn combine_err<'a, TT: TokenType, T: Token<TT>>(a: Option<ParseError<'a, TT, T>>, b: Option<ParseError<'a, TT, T>>) -> Option<ParseError<'a, TT, T>> {
    match (a, b) {
        (None, None) => None,
        (Some(e), None) => Some(e),
        (None, Some(e)) => Some(e),
        (Some(e1), Some(e2)) => Some(e1.combine(e2))
    }
}

pub type ParseResult<'a, TT, T> = Result<ParseSuccess<'a, TT, T>, ParseError<'a, TT, T>>;
