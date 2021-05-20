use crate::lexer::lexer::{LexerItem};
use crate::parser::parser::*;
use crate::parser::parser_expression::*;
use std::fmt::{Display, Formatter};

#[derive(Debug, Eq, PartialEq)]
pub enum Statement<'a> {
    Expression(Expression<'a>),
}

impl<'a> Statement<'a> {
    pub fn print(&self) {
        match self {
            Statement::Expression(e) => {
                e.print_indented(0);
                println!();
            }
        }
    }
}

pub fn parse_statement<'a>(input: &'a [LexerItem<'a>]) -> Result<(Statement<'a>, &'a [LexerItem<'a>]), ParseError> {
    alt(input, vec![
        |input| parse_expression(input).map(|(v, r)| (Statement::Expression(v), r))
    ])
}
