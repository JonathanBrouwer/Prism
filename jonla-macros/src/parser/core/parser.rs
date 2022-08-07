use crate::parser::core::error::ParseError;
use crate::parser::core::presult::PResult;
use crate::parser::core::stream::Stream;

pub trait Parser<I: Clone + Eq, O, S: Stream<I = I>, E: ParseError, Q> {
    fn parse(&self, stream: S, state: &mut Q) -> PResult<O, E, S>;
}

impl<
        I: Clone + Eq,
        O,
        S: Stream<I = I>,
        E: ParseError,
        Q,
        T: Fn(S, &mut Q) -> PResult<O, E, S>,
    > Parser<I, O, S, E, Q> for T
{
    #[inline(always)]
    fn parse(&self, stream: S, state: &mut Q) -> PResult<O, E, S> {
        self(stream, state)
    }
}
