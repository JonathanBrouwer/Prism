use crate::grammar::CharClass;
use itertools::Itertools;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};

/// Represents a parser that parsed its value successfully.
/// It parsed the value of type `O`.
/// It also stores the best error encountered during parsing, and the position AFTER the parsed value in `pos`.
#[derive(Clone, Debug)]
pub struct ParseResult<'grm, O: Clone> {
    pub inner: Result<ParseOk<'grm, O>, ParseError<'grm>>,
}

impl<'grm, O: Clone> ParseResult<'grm, O> {
    pub fn add_error_info(&mut self, error: ParseError<'grm>) {
        match &mut self.inner {
            Ok(ok) => match &mut ok.best_error {
                n @ None => *n = Some(error),
                Some(err) => err.combine_mut(error),
            },
            Err(err) => {
                err.combine_mut(error);
            }
        }
    }

    pub fn map<F, ON: Clone>(self, mapfn: F) -> ParseResult<'grm, ON>
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

    pub fn map_with_pos<F, ON: Clone>(self, mapfn: F) -> ParseResult<'grm, ON>
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

    pub fn map_errs<F>(self, mapfn: F) -> ParseResult<'grm, O>
    where
        F: FnOnce(ParseError<'grm>) -> ParseError<'grm>,
    {
        ParseResult {
            inner: self.inner.map_err(|err| mapfn(err)),
        }
    }

    pub fn new_ok(result: O, pos: usize) -> Self {
        ParseResult {
            inner: Ok(ParseOk {
                result,
                best_error: None,
                pos,
            }),
        }
    }
    pub fn new_ok_with_err(result: O, pos: usize, best_error: Option<ParseError<'grm>>) -> Self {
        ParseResult {
            inner: Ok(ParseOk {
                result,
                best_error,
                pos,
            }),
        }
    }

    pub fn new_err(pos: usize, labels: Vec<ParseErrorLabel<'grm>>) -> Self {
        ParseResult {
            inner: Err(ParseError {
                labels,
                pos,
                start: None,
                left_recursion_warning: false,
            }),
        }
    }

    pub fn new_err_leftrec(pos: usize) -> Self {
        ParseResult {
            inner: Err(ParseError {
                labels: vec![],
                pos,
                start: None,
                left_recursion_warning: true,
            }),
        }
    }

    pub fn from_ok(ok: ParseOk<'grm, O>) -> Self {
        ParseResult { inner: Ok(ok) }
    }

    pub fn from_err(err: ParseError<'grm>) -> Self {
        ParseResult { inner: Err(err) }
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
pub struct ParseOk<'grm, O: Clone> {
    pub result: O,
    pub best_error: Option<ParseError<'grm>>,
    pub pos: usize,
}

#[derive(Clone, Debug)]
pub struct ParseError<'grm> {
    pub labels: Vec<ParseErrorLabel<'grm>>,
    pub pos: usize,
    pub start: Option<usize>,
    pub left_recursion_warning: bool,
}

impl<'grm> ParseError<'grm> {
    pub fn combine_mut(&mut self, mut other: ParseError<'grm>) {
        match self.pos.cmp(&other.pos) {
            Ordering::Less => *self = other,
            Ordering::Greater => {}
            Ordering::Equal => {
                self.labels.append(&mut other.labels);
            }
        }
    }

    pub fn combine(mut self, other: ParseError<'grm>) -> ParseError<'grm> {
        self.combine_mut(other);
        self
    }

    pub fn combine_option_parse_error(
        a: Option<ParseError<'grm>>,
        b: Option<ParseError<'grm>>,
    ) -> Option<ParseError<'grm>> {
        match (a, b) {
            (None, None) => None,
            (None, Some(e)) => Some(e),
            (Some(e), None) => Some(e),
            (Some(e1), Some(e2)) => Some(e1.combine(e2)),
        }
    }
}

#[derive(Clone, Debug)]
pub enum ParseErrorLabel<'grm> {
    CharClass(CharClass),
    /// No attempt was even made
    RemainingInputNotParsed,
    Error(&'grm str),
}

impl Display for ParseErrorLabel<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseErrorLabel::CharClass(cc) => {
                fn show_char(c: char) -> String {
                    match c {
                        '\t' => "\\t".to_string(),
                        '\n' => "\\n".to_string(),
                        ' ' => "' '".to_string(),
                        c => c.to_string(),
                    }
                }
                write!(
                    f,
                    "{}",
                    cc.ranges
                        .iter()
                        .map(|(s, e)| {
                            if *s == *e {
                                format!("{}", show_char(*s))
                            } else {
                                format!("{}-{}", show_char(*s), show_char(*e))
                            }
                        })
                        .format(" ")
                )
            }
            ParseErrorLabel::RemainingInputNotParsed => {
                write!(f, "No Parse Attempt")
            }
            ParseErrorLabel::Error(err) => {
                write!(f, "{}", err)
            }
        }
    }
}

impl<'grm> ParseError<'grm> {
    pub fn display(&self, src: &str) {
        let start = self.start.unwrap_or(self.pos);
        let mut start_nl = start;
        while let Some(c) = src[..start_nl].chars().rev().next() {
            if c == '\n' {
                break;
            }
            start_nl -= c.len_utf8();
        }

        let (end_excl, end_excl_nl) = if let Some(c) = src[self.pos..].chars().next() {
            let end_excl = self.pos + c.len_utf8();
            let mut end_excl_nl = end_excl;
            while let Some(c) = src[end_excl_nl..].chars().next() {
                if c == '\n' {
                    break;
                }
                end_excl_nl += c.len_utf8();
            }
            (end_excl, end_excl_nl)
        } else {
            (self.pos + 1, self.pos)
        };

        println!("{}", &src[start_nl..end_excl_nl]);
        print!("{: <1$}", "", start - start_nl);
        println!("{:^<1$}", "", end_excl - start);

        if self.labels.len() > 0 {
            println!("Expected: {}", self.labels.iter().format(", "))
        } else if self.left_recursion_warning {
            println!("Warning: Left recursion failed here.")
        }
    }
}
