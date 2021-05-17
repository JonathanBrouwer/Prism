use crate::parser::parser::JonlaParser;
use crate::lexer::lexer::LexerTokenType;
use std::fmt::{Display, Formatter};
use std::fmt;
use crate::parser::parser_expr::Expression;

#[derive(Debug)]
pub struct FunctionDefinition<'a> {
    name: &'a str,
}

impl<'a> JonlaParser<'a> {
    pub fn parse_expr_function(&mut self) -> Result<FunctionDefinition<'a>, String> {
        self.expect_keyword("fn")?;
        let name = self.expect_identifier()?;
        self.expect_keyword(":")?;
        self.parse

        let name = self.expect_identifier()?;

        Ok(FunctionDefinition{name})
    }
}
