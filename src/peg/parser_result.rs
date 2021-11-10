use std::cmp::Ordering;
use std::fmt::{Debug, Display};
use crate::PegRuleResult;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct ParseSuccess<IE: Debug + Display + PartialEq + Eq + Clone + Copy> {
    pub result: PegRuleResult<IE>,
    pub best_error: Option<ParseError<IE>>,
    pub rest: usize,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct ParseError<IE: Debug + Display + PartialEq + Eq + Clone + Copy> {
    pub positives: Vec<IE>,
    pub flags: Vec<ParseErrorFlag>,
    pub location: usize,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum ParseErrorFlag {
    Recursive,
    NotAllInput,
}

impl<IE: Debug + Display + PartialEq + Eq + Clone + Copy> ParseError<IE> {
    pub(crate) fn parse_error_combine_opt2(e1: Option<ParseError<IE>>, e2: Option<ParseError<IE>>) -> Option<ParseError<IE>> {
        match (e1, e2) {
            (Some(e1), Some(e2)) => Some(Self::parse_error_combine(e1, e2)),
            (Some(e1), None) => Some(e1),
            (None, Some(e2)) => Some(e2),
            (None, None) => None,
        }
    }

    pub(crate) fn parse_error_combine_opt1(e1: ParseError<IE>, e2: Option<ParseError<IE>>) -> ParseError<IE> {
        match e2 {
            Some(e2) => Self::parse_error_combine(e1, e2),
            None => e1
        }
    }

    pub(crate) fn parse_error_combine(mut e1: ParseError<IE>, mut e2: ParseError<IE>) -> ParseError<IE> {
        match e1.location.cmp(&e2.location) {
            Ordering::Less => e2,
            Ordering::Greater => e1,
            Ordering::Equal => {
                e1.positives.append(&mut e2.positives);
                e1.flags.append(&mut e2.flags);
                e1
            }
        }
    }
}