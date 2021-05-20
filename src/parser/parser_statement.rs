use crate::lexer::lexer::{LexerItem};
use crate::parser::parser::*;
use crate::parser::parser_expression::*;

#[derive(Debug, Eq, PartialEq)]
pub enum Statement<'a> {
    Expression(Expression<'a>),
}

pub fn parse_statement<'a>(input: &'a [LexerItem<'a>]) -> Result<(Statement<'a>, &'a [LexerItem<'a>]), ParseError> {
    alt(input, vec![
        |input| parse_expression(input).map(|(v, r)| (Statement::Expression(v), r))
    ])
}
