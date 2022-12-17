use crate::core::context::{PCache, ParserContext};
use crate::error::ParseError;
use crate::core::presult::PResult;
use crate::core::stream::StringStream;

pub trait Parser<'b, 'grm, O, E: ParseError> {
    fn parse(
        &self,
        stream: StringStream<'grm>,
        cache: &mut PCache<'b, 'grm, E>,
        context: &ParserContext<'b, 'grm>,
    ) -> PResult<'grm, O, E>;
}

impl<
        'b,
        'grm,
        O,
        E: ParseError,
        T: Fn(
            StringStream<'grm>,
            &mut PCache<'b, 'grm, E>,
            &ParserContext<'b, 'grm>,
        ) -> PResult<'grm, O, E>,
    > Parser<'b, 'grm, O, E> for T
{
    #[inline(always)]
    fn parse(
        &self,
        stream: StringStream<'grm>,
        cache: &mut PCache<'b, 'grm, E>,
        context: &ParserContext<'b, 'grm>,
    ) -> PResult<'grm, O, E> {
        self(stream, cache, context)
    }
}
