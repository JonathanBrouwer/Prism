use crate::parser::core::error::ParseError;
use crate::parser::core::presult::PResult;
use crate::parser::core::stream::Stream;

pub trait Parser<O, S: Stream, E: ParseError, Q> {
    fn parse(&self, stream: S, state: &mut Q) -> PResult<O, E, S>;
}

impl<O, S: Stream, E: ParseError, Q, T: Fn(S, &mut Q) -> PResult<O, E, S>> Parser<O, S, E, Q>
    for T
{
    #[inline(always)]
    fn parse(&self, stream: S, state: &mut Q) -> PResult<O, E, S> {
        self(stream, state)
    }
}
