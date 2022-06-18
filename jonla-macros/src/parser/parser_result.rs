use std::cmp::Ordering;
use crate::grammar::CharClass;

/// Represents a parser that parsed its value successfully.
/// It parsed the value of type `O`.
/// It also stores the best error encountered during parsing, and the position AFTER the parsed value in `pos`.
#[derive(Clone, Debug)]
pub struct ParseResult<O: Clone> {
    pub inner: Result<ParseOk<O>, ParseError>,
}

impl<O: Clone> ParseResult<O> {
    pub fn map<F, ON: Clone>(self, mapfn: F) -> ParseResult<ON>
    where
        F: FnOnce(O) -> ON,
    {
        ParseResult {
            inner: self.inner.map(|ok| ParseOk {
                result: mapfn(ok.result),
                best_error: ok.best_error,
                pos: ok.pos,
            }),
        }
    }

    pub fn map_with_pos<F, ON: Clone>(self, mapfn: F) -> ParseResult<ON>
        where
            F: FnOnce(O, usize) -> ON,
    {
        ParseResult {
            inner: self.inner.map(|ok| ParseOk {
                result: mapfn(ok.result, ok.pos),
                best_error: ok.best_error,
                pos: ok.pos,
            }),
        }
    }

    pub fn new_ok(result: O, pos: usize) -> Self {
        ParseResult {
            inner: Ok(ParseOk { result, best_error: None, pos }),
        }
    }
    pub fn new_ok_with_err(result: O, pos: usize, best_error: Option<ParseError>) -> Self {
        ParseResult {
            inner: Ok(ParseOk { result, best_error, pos }),
        }
    }

    pub fn new_err(pos: usize, labels: Vec<ParseErrorLabel>) -> Self {
        ParseResult {
            inner: Err(ParseError { labels, pos }),
        }
    }

    pub fn from_ok(ok: ParseOk<O>) -> Self {
        ParseResult {
            inner: Ok(ok),
        }
    }

    pub fn from_err(err: ParseError) -> Self {
        ParseResult {
            inner: Err(err),
        }
    }

    pub fn is_ok(&self) -> bool {
        self.inner.is_ok()
    }

    pub fn pos(&self) -> usize {
        match &self.inner {
            Ok(ok) => ok.pos,
            Err(err) => err.pos,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ParseOk<O: Clone> {
    pub result: O,
    pub best_error: Option<ParseError>,
    pub pos: usize,
}

#[derive(Clone, Debug)]
pub struct ParseError {
    pub labels: Vec<ParseErrorLabel>,
    pub pos: usize,
}

impl ParseError {
    pub fn combine(mut self, mut other: ParseError) -> ParseError {
        match self.pos.cmp(&other.pos) {
            Ordering::Less => other,
            Ordering::Greater => self,
            Ordering::Equal => {
                self.labels.append(&mut other.labels);
                self
            }
        }
    }

    pub fn combine_option_parse_error(
        a: Option<ParseError>,
        b: Option<ParseError>,
    ) -> Option<ParseError> {
        match (a, b) {
            (None, None) => None,
            (None, Some(e)) => Some(e),
            (Some(e), None) => Some(e),
            (Some(e1), Some(e2)) => Some(e1.combine(e2)),
        }
    }
}

#[derive(Clone, Debug)]
pub enum ParseErrorLabel {
    CharClass(CharClass),
    LeftRecursionWarning,
    /// No attempt was even made
    RemainingInputNotParsed,
}
