use crate::parser_core::error::ParseError;
use crate::parser_core::presult::PResult;
use crate::parser_core::stream::StringStream;
use crate::parser_sugar::parser_context::{ParserContext, PCache};

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
        T: Fn(StringStream<'grm>, &mut PCache<'b, 'grm, E>, &ParserContext<'b, 'grm>) -> PResult<'grm, O, E>,
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
