use crate::parser::core::error::ParseError;
use crate::parser::core::presult::PResult;
use crate::parser::core::stream::StringStream;

pub trait Parser<'grm, O, E: ParseError, Q> {
    fn parse(&self, stream: StringStream<'grm>, state: &mut Q) -> PResult<'grm, O, E>;
}

impl<'grm, O, E: ParseError, Q, T: Fn(StringStream<'grm>, &mut Q) -> PResult<'grm, O, E>> Parser<'grm, O, E, Q>
    for T
{
    #[inline(always)]
    fn parse(&self, stream: StringStream<'grm>, state: &mut Q) -> PResult<'grm, O, E> {
        self(stream, state)
    }
}
