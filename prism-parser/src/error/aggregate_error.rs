use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use ariadne::Source;
use std::io;

pub struct AggregatedParseError<'p, E: ParseError<L = ErrorLabel<'p>> + 'p> {
    pub input: &'p str,
    pub errors: Vec<E>,
}

impl<'p, E: ParseError<L = ErrorLabel<'p>> + 'p> AggregatedParseError<'p, E> {
    pub fn eprint(&self) -> io::Result<()> {
        for e in &self.errors {
            e.report(false).eprint(Source::from(self.input))?
        }
        Ok(())
    }
}

pub trait ParseResultExt<T> {
    fn unwrap_or_eprint(self) -> T;
}

impl<'p, E: ParseError<L = ErrorLabel<'p>> + 'p, T> ParseResultExt<T>
    for Result<T, AggregatedParseError<'p, E>>
{
    fn unwrap_or_eprint(self) -> T {
        self.unwrap_or_else(|es| {
            es.eprint().unwrap();
            panic!("Failed to parse grammar")
        })
    }
}
