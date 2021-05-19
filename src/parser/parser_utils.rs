use crate::lexer::lexer::{LexerItem, LexerToken};
use crate::parser::parser::ParseError;
use crate::lexer::lexer::LexerToken::Name;

pub fn take_one<'a>(input: &'a [LexerItem<'a>]) -> Option<(&'a LexerItem<'a>, &'a [LexerItem<'a>])> {
    if input.len() > 0 {
        Some((&input[0], &input[1..]))
    } else {
        None
    }
}

pub fn expect<'a>(input: &'a [LexerItem<'a>], token: LexerToken) -> Result<(&'a LexerItem<'a>, &'a [LexerItem<'a>]), ParseError> {
    match take_one(input) {
        None => Err(ParseError::ParseError),
        Some((v, rest)) => if v.token == token {
            Ok((v, rest))
        } else {
            Err(ParseError::ParseError)
        }
    }
}

pub fn expect_name<'a>(input: &'a [LexerItem<'a>]) -> Result<(&'a str, &'a [LexerItem<'a>]), ParseError> {
    match take_one(input) {
        None => Err(ParseError::ParseError),
        Some((v, rest)) => match v {
            LexerItem { token: Name(name), .. } => Ok((name, rest)),
            item => Err(ParseError::ParseError)
        }
    }
}

pub fn expect_keyword<'a>(input: &'a [LexerItem<'a>], keyword: &'static str) -> Result<&'a [LexerItem<'a>], ParseError> {
    let (name, rest) = expect_name(input)?;
    if name == keyword {
        Ok(rest)
    } else {
        Err(ParseError::ParseError)
    }
}

pub fn alt<'a, T>(input: &'a [LexerItem<'a>], parsers: Vec<fn(&'a [LexerItem<'a>]) -> Result<(T, &'a [LexerItem<'a>]), ParseError>>)
                  -> Result<(T, &'a [LexerItem<'a>]), ParseError> {
    assert!(parsers.len() > 0);
    let mut error: Option<ParseError> = None;
    for parser in parsers {
        match parser(input) {
            Ok(v) => return Ok(v),
            Err(e2) => error = match error {
                Some(e1) => Some(e1.combine(e2)),
                None => Some(e2),
            }
        }
    }
    return Err(error.unwrap());
}

pub fn times<'a, T>(mut input: &'a [LexerItem<'a>], parser: fn(&'a [LexerItem<'a>]) -> Result<(T, &'a [LexerItem<'a>]), ParseError>,
                    min: Option<usize>, max: Option<usize>)
                    -> Result<(Vec<T>, &'a [LexerItem<'a>]), ParseError> {
    let mut result = Vec::new();

    //Do minimum amount of times
    for _ in 0..(min.unwrap_or(0)) {
        let (res, rest) = parser(input)?;
        result.push(res);
        input = rest;
    }

    //Do from minimum to maximum amount of times
    for _ in (min.unwrap_or(0))..(max.unwrap_or(usize::MAX)) {
        let (res, rest) = match parser(input) {
            Ok(v) => v,
            Err(_) => return Ok((result, input))
        };
        result.push(res);
        input = rest;
    }

    return Ok((result, input));
}

pub fn zero_or_more<'a, T>(input: &'a [LexerItem<'a>], parser: fn(&'a [LexerItem<'a>]) -> Result<(T, &'a [LexerItem<'a>]), ParseError>)
                           -> (Vec<T>, &'a [LexerItem<'a>]) {
    times(input, parser, None, None).unwrap()
}

pub fn one_or_more<'a, T>(input: &'a [LexerItem<'a>], parser: fn(&'a [LexerItem<'a>]) -> Result<(T, &'a [LexerItem<'a>]), ParseError>)
                           -> Result<(Vec<T>, &'a [LexerItem<'a>]), ParseError> {
    times(input, parser, Some(1), None)
}

pub fn zero_or_one<'a, T>(input: &'a [LexerItem<'a>], parser: fn(&'a [LexerItem<'a>]) -> Result<(T, &'a [LexerItem<'a>]), ParseError>)
                          -> (Option<T>, &'a [LexerItem<'a>]) {
    match parser(input) {
        Ok((v, rest)) => (Some(v), rest),
        Err(_) => return (None, input)
    }
}
