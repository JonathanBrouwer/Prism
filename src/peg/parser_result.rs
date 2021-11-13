use std::cmp::Ordering;
use std::fmt::{Debug, Display};
use crate::peg::rules::PegRuleResult;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct ParseSuccess<IE: Debug + Display + PartialEq + Eq + Clone + Copy, ER: Debug + Display + PartialEq + Eq + Clone + Copy> {
    pub result: PegRuleResult<IE>,
    pub best_error: Option<ParseError<ER>>,
    pub rest: usize,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct ParseError<ER: Debug + Display + PartialEq + Eq + Clone + Copy> {
    pub positives: Vec<ER>,
    pub flags: Vec<ParseErrorFlag>,
    pub location: usize,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum ParseErrorFlag {
    Recursive,
    NotAllInput,
}

impl<ER: Debug + Display + PartialEq + Eq + Clone + Copy> ParseError<ER> {
    pub(crate) fn parse_error_combine_opt2(e1: Option<ParseError<ER>>, e2: Option<ParseError<ER>>) -> Option<ParseError<ER>> {
        match (e1, e2) {
            (Some(e1), Some(e2)) => Some(Self::parse_error_combine(e1, e2)),
            (Some(e1), None) => Some(e1),
            (None, Some(e2)) => Some(e2),
            (None, None) => None,
        }
    }

    pub(crate) fn parse_error_combine_opt1(e1: ParseError<ER>, e2: Option<ParseError<ER>>) -> ParseError<ER> {
        match e2 {
            Some(e2) => Self::parse_error_combine(e1, e2),
            None => e1
        }
    }

    pub(crate) fn parse_error_combine(mut e1: ParseError<ER>, mut e2: ParseError<ER>) -> ParseError<ER> {
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