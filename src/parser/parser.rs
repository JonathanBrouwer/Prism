use crate::lexer::lexer::{LexerItem, LexerToken};
use crate::parser::parser_utils::*;
use std::ops::Range;
use crate::lexer::lexer::LexerToken::{Line, Name};
use crate::parser::parser::ParseError::LeftoverJunk;

#[derive(Debug, Eq, PartialEq)]
pub struct ProgramFile {
    pub statements: Vec<Statement>
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum ParseError {
    ParseError,
    LeftoverJunk
}

impl ParseError {
    pub fn combine(self, other: ParseError) -> ParseError {
        todo!()
    }
}

pub fn parse_program_file_final<'a>(mut input: &'a [LexerItem<'a>]) -> Result<ProgramFile, ParseError> {
    let (program, rest) = parse_program_file(input)?;
    if rest.len() == 0 {
        Ok(program)
    } else {
        Err(LeftoverJunk)
    }
}

pub fn parse_program_file<'a>(mut input: &'a [LexerItem<'a>]) -> Result<(ProgramFile, &'a [LexerItem<'a>]), ParseError> {
    let mut statements = Vec::new();

    loop {
        if let Ok((v, rest)) = parse_statement(input) {
            statements.push(v);
            input = rest;
        } else {
            return Ok((ProgramFile{ statements }, input))
        }
        if let Ok((_, rest)) = expect(input, Line) {
            input = rest;
        } else {
            return Ok((ProgramFile{ statements }, input))
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Statement {

}

pub fn parse_statement<'a>(input: &'a [LexerItem<'a>]) -> Result<(Statement, &'a [LexerItem<'a>]), ParseError> {
    let (_, rest) = expect_name(input)?;
    Ok((Statement{}, rest))
}
