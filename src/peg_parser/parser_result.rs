
use std::cmp::Ordering;
use crate::peg_parser::parser_token::*;

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum Expected<TT: TokenType, TV: TokenValue> {
    LiteralExact(TV),
    LiteralBind(TT),
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum ParseError<TT: TokenType, TV: TokenValue, T: Token<TT, TV>> {
    Expect { on: T, expect: Vec<Expected<TT, TV>>, inv_priority: usize },
    Recursion
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct ParseErrors<TT: TokenType, TV: TokenValue, T: Token<TT, TV>> {
    errors: Vec<ParseError<TT, TV, T>>
}

impl<'a, TT: TokenType, TV: TokenValue, T: Token<TT, TV>> ParseErrors<TT, TV, T> {
    pub fn new(error: ParseError<TT, TV, T>) -> ParseErrors<TT, TV, T> {
        Self { errors: vec![error] }
    }

    pub fn combine(mut self, mut other: ParseErrors<TT, TV, T>) -> ParseErrors<TT, TV, T> {
        self.errors.append(&mut other.errors);
        self
    }
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
    pub(crate) best_error: Option<ParseErrors<TT, TV, T>>,
    pub(crate) rest: &'a [T],
}



pub fn combine_err<'a, TT: TokenType, TV: TokenValue, T: Token<TT, TV>>(a: Option<ParseErrors<TT, TV, T>>, b: Option<ParseErrors<TT, TV, T>>) -> Option<ParseErrors<TT, TV, T>> {
    match (a, b) {
        (None, None) => None,
        (Some(e), None) => Some(e),
        (None, Some(e)) => Some(e),
        (Some(e1), Some(e2)) => Some(e1.combine(e2))
    }
}

pub type ParseResult<'a, TT, TV, T> = Result<ParseSuccess<'a, TT, TV, T>, ParseErrors<TT, TV, T>>;
