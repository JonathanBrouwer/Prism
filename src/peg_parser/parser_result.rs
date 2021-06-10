
use std::cmp::Ordering;
use crate::peg_parser::parser_token::*;

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum Expected<TT: TokenType, TV: TokenValue> {
    LiteralExact(TV),
    LiteralBind(TT),
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct ParseError<'a, TT: TokenType, TV: TokenValue, T: Token<TT, TV>> {
    pub on: &'a T,
    pub expect: Vec<Expected<TT, TV>>,
    pub inv_priority: usize
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum ParseTree<T> {
    Value(T),
    Sequence(Vec<ParseTree<T>>),
    ChooseFirst(usize, Box<ParseTree<T>>),
    Empty
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct ParseSuccess<'a, TT: TokenType, TV: TokenValue, T: Token<TT, TV>> {
    pub(crate) result: ParseTree<T>,
    pub(crate) best_error: Option<ParseError<'a, TT, TV, T>>,
    pub(crate) rest: &'a [T],
}

impl<'a, TT: TokenType, TV: TokenValue, T: Token<TT, TV>> ParseError<'a, TT, TV, T> {
    pub fn combine(mut self, other: ParseError<'a, TT, TV, T>) -> ParseError<'a, TT, TV, T> {
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

pub fn combine_err<'a, TT: TokenType, TV: TokenValue, T: Token<TT, TV>>(a: Option<ParseError<'a, TT, TV, T>>, b: Option<ParseError<'a, TT, TV, T>>) -> Option<ParseError<'a, TT, TV, T>> {
    match (a, b) {
        (None, None) => None,
        (Some(e), None) => Some(e),
        (None, Some(e)) => Some(e),
        (Some(e1), Some(e2)) => Some(e1.combine(e2))
    }
}

pub type ParseResult<'a, TT, TV, T> = Result<ParseSuccess<'a, TT, TV, T>, ParseError<'a, TT, TV, T>>;
