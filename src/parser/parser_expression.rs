use crate::lexer::lexer::{LexerItem};
use crate::parser::parser::*;
use crate::lexer::lexer::LexerToken::{Line};


#[derive(Debug, Eq, PartialEq)]
pub enum Expression<'a> {
    Name(&'a str),
    ExpressionSequence(Vec<Expression<'a>>),
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
        Ok((Expression::ExpressionSequence(res.0), res.1))
    }
}

pub fn parse_expression_parens<'a>(input: &'a [LexerItem<'a>]) -> Result<(Expression, &'a [LexerItem<'a>]), ParseError> {
    let input = expect_control_keyword(input, "(")?;
    let (expr, input) = parse_expression(input)?;
    let input = expect_control_keyword(input, ")")?;
    Ok((expr, input))
}

pub fn parse_expression_lookupname<'a>(input: &'a [LexerItem<'a>]) -> Result<(Expression, &'a [LexerItem<'a>]), ParseError> {
    let (name, input) = expect_name(input)?;
    Ok((Expression::Name(name), input))
}

#[derive(Debug, Eq, PartialEq)]
pub struct Argument<'a> {
    names: Vec<&'a str>,
    argtype: Expression<'a>
}

pub fn parse_arguments<'a>(input: &'a [LexerItem<'a>]) -> Result<(Vec<Argument>, &'a [LexerItem<'a>]), ParseError> {
    Ok(zero_or_more(input, |input| {
        let input = expect_control_keyword(input, "(")?;
        let (names, input) = one_or_more(input, |input| expect_name(input))?;
        let input = expect_name_keyword(input, ":")?;
        let (argtype, input) = parse_expression(input)?;
        let input = expect_control_keyword(input, ")")?;

        Ok((Argument {names, argtype}, input))
    }))
}