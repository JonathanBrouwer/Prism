use crate::lexer::lexer::{LexerItem, LexerToken, LexerTokenType};
use crate::lexer::lexer::LexerToken::{Name, Control};

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct ParseSuccess<'a, R> {
    pub(crate) result: R,
    pub(crate) best_error: Option<ParseError<'a>>,
    pub(crate) rest: &'a [LexerItem<'a>]
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct ParseError<'a> {
    pub(crate) on: &'a [LexerItem<'a>],
    pub(crate) expect: Vec<Expected<'a>>
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum Expected<'a> {
    Token(LexerToken<'a>),
    TokenType(LexerTokenType),
}

impl<'a> ParseError<'a> {
    pub fn combine(self, other: ParseError) -> ParseError {
        todo!()
    }
}

pub fn next<'a>(input: &'a [LexerItem<'a>]) -> (&'a LexerItem<'a>, &'a [LexerItem<'a>]) {
    assert!(input.len() > 0);
    (&input[0], &input[1..])
}

pub fn expect_exact<'a>(input: &'a [LexerItem<'a>], expected: LexerToken<'a>) -> Result<ParseSuccess<'a, &'a LexerItem<'a>>, ParseError<'a>> {
    let (actual, rest) = next(input);
    if actual.token == expected {
        Ok(ParseSuccess{ result: actual, rest, best_error: None })
    } else {
        Err(ParseError{ on: &input[0..1], expect: vec![ Expected::Token(expected) ] })
    }
}

pub fn expect_type<'a>(input: &'a [LexerItem<'a>], expected: LexerTokenType) -> Result<ParseSuccess<'a, &'a LexerItem<'a>>, ParseError<'a>> {
    let (actual, rest) = next(input);
    if actual.token.to_type() == expected {
        Ok(ParseSuccess{ result: actual, rest, best_error: None })
    } else {
        Err(ParseError{ on: &input[0..1], expect: vec![ Expected::TokenType(expected) ] })
    }
}

pub fn alt<'a, R>(input: &'a [LexerItem<'a>],
                  parsers: Vec<fn(&'a [LexerItem<'a>]) -> Result<ParseSuccess<'a, R>, ParseError<'a>>>)
                  -> Result<ParseSuccess<'a, R>, ParseError<'a>>{
    assert!(parsers.len() > 0);

    let mut error: Option<ParseError<'a>> = None;
    for parser in parsers {
        match parser(input) {
            Ok(v) => {
                let best_error = match (error, v.best_error) {
                    (None, None) => None,
                    (Some(e), None) => Some(e),
                    (None, Some(e)) => Some(e),
                    (Some(e1), Some(e2)) => Some(e1.combine(e2))
                };
                return Ok(ParseSuccess{result: v.result, rest: v.rest, best_error: best_error})
            },
            Err(e2) => error = match error {
                Some(e1) => Some(e1.combine(e2)),
                None => Some(e2),
            }
        }
    }
    return Err(error.unwrap());
}

// pub fn times<'a, R>(mut input: &'a [LexerItem<'a>],
//                     parser: fn(&'a [LexerItem<'a>]) -> Result<ParseSuccess<'a, R>, ParseError<'a>>,
//                     min: Option<usize>, max: Option<usize>)
//                     -> Result<ParseSuccess<'a, R>, ParseError<'a>> {
//     let mut result = Vec::new();
//
//     //Do minimum amount of times
//     for _ in 0..(min.unwrap_or(0)) {
//         let (res, rest) = parser(input)?;
//         result.push(res);
//         input = rest;
//     }
//
//     //Do from minimum to maximum amount of times
//     for _ in (min.unwrap_or(0))..(max.unwrap_or(usize::MAX)) {
//         let (res, rest) = match parser(input) {
//             Ok(v) => v,
//             Err(_) => return Ok((result, input))
//         };
//         result.push(res);
//         input = rest;
//     }
//
//     return Ok((result, input));
// }
//
// pub fn zero_or_more<'a, T>(input: &'a [LexerItem<'a>], parser: fn(&'a [LexerItem<'a>]) -> Result<(T, &'a [LexerItem<'a>]), ParseError>)
//                            -> (Vec<T>, &'a [LexerItem<'a>]) {
//     times(input, parser, None, None).unwrap()
// }
//
// pub fn one_or_more<'a, T>(input: &'a [LexerItem<'a>], parser: fn(&'a [LexerItem<'a>]) -> Result<(T, &'a [LexerItem<'a>]), ParseError>)
//                            -> Result<(Vec<T>, &'a [LexerItem<'a>]), ParseError> {
//     times(input, parser, Some(1), None)
// }
//
// pub fn maybe<'a, T>(input: &'a [LexerItem<'a>], parser: fn(&'a [LexerItem<'a>]) -> Result<(T, &'a [LexerItem<'a>]), ParseError>)
//                           -> (Option<T>, &'a [LexerItem<'a>]) {
//     match parser(input) {
//         Ok((v, rest)) => (Some(v), rest),
//         Err(_) => return (None, input)
//     }
// }
