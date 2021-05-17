use crate::parser::parser::JonlaParser;
use crate::lexer::lexer::LexerTokenType;
use std::fmt::{Display, Formatter};
use std::fmt;
use crate::parser::parser_stmt::Statement;

#[derive(Debug)]
pub struct DataDefinition<'a> {
    name: &'a str,
    constructors: Vec<DataDefinitionConstructor<'a>>
}

#[derive(Debug)]
pub struct DataDefinitionConstructor<'a> {
    name: &'a str
}

impl<'a> JonlaParser<'a> {
    pub fn parse_stmt_data(&mut self) -> Result<DataDefinition<'a>, String> {
        self.expect_keyword("data")?;
        let name = self.expect_identifier()?;
        self.expect_keyword("=")?;
        self.expect(LexerTokenType::Line)?;
        self.expect(LexerTokenType::BlockStart)?;

        let mut constructors = Vec::new();
        loop {
            let cursor_old = self.cursor;
            let cons = self.parse_data_constructor();
            if let Ok(cons) = cons {
                constructors.push(cons);
            } else {
                self.cursor = cursor_old;
                break;
            }
        }

        self.expect(LexerTokenType::BlockStop)?;

        Ok(DataDefinition{name, constructors})
    }

    pub fn parse_data_constructor(&mut self) -> Result<DataDefinitionConstructor<'a>, String> {
        let name = self.expect_identifier()?;
        self.expect(LexerTokenType::Line)?;
        Ok(DataDefinitionConstructor{name})
    }
}

