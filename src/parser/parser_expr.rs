use crate::parser::parser::JonlaParser;
use crate::lexer::lexer::LexerTokenType;
use std::fmt::{Display, Formatter};
use std::fmt;
use crate::parser::parser_expr_functiondefinition::FunctionDefinition;

#[derive(Debug)]
pub enum Expression<'a> {
    FunctionDefinition(FunctionDefinition<'a>)
}

impl<'a> JonlaParser<'a> {
    pub fn parse_expr(&mut self) -> Result<Expression<'a>, String> {
        self.or(vec![
            |p| Self::parse_expr_function(p).map(Expression::FunctionDefinition),
        ])
    }
}


