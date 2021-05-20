use crate::lexer::lexer::{LexerItem, LexerToken};
use crate::parser::parser::*;
use crate::lexer::lexer::LexerToken::*;


#[derive(Debug, Eq, PartialEq)]
pub enum Expression<'a> {
    Name(&'a str),
    ExpressionSequence(Vec<Expression<'a>>),
    Block(Vec<Expression<'a>>),
}

impl<'a> Expression<'a> {
    pub fn print_indented(&self, indent: usize) {
        match self {
            Expression::Name(n) => print!("{}", n),
            Expression::ExpressionSequence(vec) => for exp in vec {
                exp.print_indented(indent);
                print!(" ");
            }
            Expression::Block(vec) => {

                for (n, exp) in vec.iter().enumerate() {
                    println!();
                    let indent = indent + 2;
                    print!("{:indent$}", "", indent=indent);
                    exp.print_indented(indent);
                }
            }
        }
    }
}

pub fn parse_expression<'a>(input: &'a [LexerItem<'a>]) -> Result<(Expression, &'a [LexerItem<'a>]), ParseError> {
    let (mut res, mut input) = one_or_more(input, |input|
        alt(input, vec![
            |input| parse_expression_parens(input),
            |input| parse_expression_name(input),
        ])
    )?;
    //Block can only occur at the end of the sequence
    if let Ok((nr, ni)) = parse_expression_block(input) {
        res.push(nr);
        input = ni;
    }
    if res.len() == 1 {
        Ok((res.into_iter().next().unwrap(), input))
    } else {
        Ok((Expression::ExpressionSequence(res), input))
    }
}

pub fn parse_expression_parens<'a>(input: &'a [LexerItem<'a>]) -> Result<(Expression, &'a [LexerItem<'a>]), ParseError> {
    let input = expect_control_keyword(input, "(")?;
    let (expr, input) = parse_expression(input)?;
    let input = expect_control_keyword(input, ")")?;
    Ok((expr, input))
}

pub fn parse_expression_name<'a>(input: &'a [LexerItem<'a>]) -> Result<(Expression, &'a [LexerItem<'a>]), ParseError> {
    let (name, input) = expect_name(input)?;
    Ok((Expression::Name(name), input))
}

pub fn parse_expression_block<'a>(input: &'a [LexerItem<'a>]) -> Result<(Expression, &'a [LexerItem<'a>]), ParseError> {
    let (_, input) = expect(input,  Line)?;
    let (_, mut input) = expect(input,  BlockStart)?;

    let mut expressions = Vec::new();
    let mut first = true;
    loop {
        if first {
            if let Ok((v, rest)) = parse_expression(input) {
                expressions.push(v);
                input = rest;
            } else {
                break
            }
        } else {
            if let Ok((_, rest)) = expect(input, LexerToken::Line) {
                input = rest;
            }
        }
        first = !first;
    }
    let (_, input) = expect(input,  BlockStop)?;
    Ok((Expression::Block(expressions), input))
}
