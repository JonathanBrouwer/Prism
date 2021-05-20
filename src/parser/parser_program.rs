use crate::lexer::lexer::{LexerItem};
use crate::parser::parser::*;
use crate::lexer::lexer::LexerToken::{Line};
use crate::parser::parser_statement::*;

#[derive(Debug, Eq, PartialEq)]
pub struct ProgramFile<'a> {
    pub statements: Vec<Statement<'a>>
}

impl<'a> ProgramFile<'a> {
    pub fn print(&self) {
        for statement in &self.statements {
            statement.print();
        }
    }
}

pub fn parse_program_file<'a>(mut input: &'a [LexerItem<'a>]) -> Result<ProgramFile<'a>, ParseError> {
    let mut statements = Vec::new();

    let mut first = true;
    while input.len() > 0 {
        if first {
            let (v, rest) = parse_statement(input)?;
            statements.push(v);
            input = rest;
        } else {
            let (_, rest) = expect(input, Line)?;
            input = rest;
        }
        first = !first;
    }

    Ok(ProgramFile { statements })
}