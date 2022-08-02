use crate::parser::core::error::ParseError;
use crate::parser::core::presult::PResult;
use crate::parser::core::stream::Stream;

pub trait Parser<I: Clone + Eq, O, S: Stream<I = I>, E: ParseError, Q> {
    fn parse(&self, stream: S, state: &mut Q) -> PResult<O, E, S>;
}

// impl<I: Clone + Eq, O, S: Stream<I = I>, E: ParseError, Q, P: Parser<I, O, S, E, Q>>
//     Parser<I, O, S, E, Q> for &P
// {
//     fn parse(&self, stream: S, state: &mut Q) -> PResult<O, E, S> {
//         self.deref().parse(stream, state)
//     }
// }
//
// impl<I: Clone + Eq, O, S: Stream<I = I>, E: ParseError, Q, P: Parser<I, O, S, E, Q>>
//     Parser<I, O, S, E, Q> for Box<P>
// {
//     fn parse(&self, stream: S, state: &mut Q) -> PResult<O, E, S> {
//         self.deref().parse(stream, state)
//     }
// }

impl<
        I: Clone + Eq,
        O,
        S: Stream<I = I>,
        E: ParseError,
        Q,
        T: Fn(S, &mut Q) -> PResult<O, E, S>,
    > Parser<I, O, S, E, Q> for T
{
    fn parse(&self, stream: S, state: &mut Q) -> PResult<O, E, S> {
        self(stream, state)
    }
}
