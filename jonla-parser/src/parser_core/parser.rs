use crate::parser_core::error::ParseError;
use crate::parser_core::presult::PResult;
use crate::parser_core::stream::StringStream;
use crate::parser_sugar::parser_rule::ParserContext;

pub trait Parser<'b, 'grm, O, E: ParseError, Q> {
    fn parse(&self, stream: StringStream<'grm>, cache: &mut Q, context: &ParserContext<'b, 'grm>) -> PResult<'grm, O, E>;
}

impl<'b, 'grm, O, E: ParseError, Q, T: Fn(StringStream<'grm>, &mut Q, &ParserContext<'b, 'grm>) -> PResult<'grm, O, E>>
    Parser<'b, 'grm, O, E, Q> for T
{
    #[inline(always)]
    fn parse(&self, stream: StringStream<'grm>, cache: &mut Q, context: &ParserContext<'b, 'grm>) -> PResult<'grm, O, E> {
        self(stream, cache, context)
    }
}
