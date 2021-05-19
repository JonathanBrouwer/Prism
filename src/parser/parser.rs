use crate::lexer::lexer::{LexerItem};
use crate::parser::parser_utils::*;
use crate::lexer::lexer::LexerToken::{Line};
use crate::parser::parser::Expression::ExpressionSequence;

#[derive(Debug, Eq, PartialEq)]
pub struct ProgramFile<'a> {
    pub statements: Vec<Statement<'a>>
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum ParseError {
    ParseError
}

impl ParseError {
    pub fn combine(self, other: ParseError) -> ParseError {
        ParseError::ParseError
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

#[derive(Debug, Eq, PartialEq)]
pub enum Statement<'a> {
    Expression(Expression<'a>)
}

pub fn parse_statement<'a>(input: &'a [LexerItem<'a>]) -> Result<(Statement<'a>, &'a [LexerItem<'a>]), ParseError> {
    alt(input, vec![
        |input| parse_expression(input).map(|(v, r)| (Statement::Expression(v), r))
    ])
}

#[derive(Debug, Eq, PartialEq)]
pub enum Expression<'a> {
    LookupName(&'a str),
    ExpressionSequence(Vec<Expression<'a>>)
}

pub fn parse_expression<'a>(input: &'a [LexerItem<'a>]) -> Result<(Expression, &'a [LexerItem<'a>]), ParseError> {
    let res = one_or_more(input, |input|
        alt(input, vec![
            |input| parse_expression_parens(input),
            |input| parse_expression_lookupname(input)
        ])
    )?;
    if res.0.len() == 1 {
        Ok((res.0.into_iter().next().unwrap(), res.1))
    } else {
        Ok((ExpressionSequence(res.0), res.1))
    }
}

pub fn parse_expression_parens<'a>(input: &'a [LexerItem<'a>]) -> Result<(Expression, &'a [LexerItem<'a>]), ParseError> {
    let input = expect_keyword(input, "(")?;
    let (expr, input) = parse_expression(input)?;
    let input = expect_keyword(input, ")")?;
    Ok((expr, input))
}

pub fn parse_expression_lookupname<'a>(input: &'a [LexerItem<'a>]) -> Result<(Expression, &'a [LexerItem<'a>]), ParseError> {
    let (name, input) = expect_name(input)?;
    Ok((Expression::LookupName(name), input))
}