use crate::core::context::{PCache, ParserContext};
use crate::core::presult::PResult;
use crate::core::stream::StringStream;
use crate::error::ParseError;

pub trait Parser<'b, 'grm, O, E: ParseError> {
    fn parse(
        &self,
        stream: StringStream<'grm>,
        cache: &mut PCache<'b, 'grm, E>,
        context: &ParserContext<'b, 'grm>,
    ) -> PResult<'grm, O, E>;
}

pub fn map_parser<'a, 'b: 'a, 'grm: 'b, O, P, E: ParseError>(
    p: impl Parser<'b, 'grm, O, E> + 'a,
    f: &'a impl Fn(O) -> P,
) -> impl Parser<'b, 'grm, P, E> + 'a {
    move |stream: StringStream<'grm>,
          cache: &mut PCache<'b, 'grm, E>,
          context: &ParserContext<'b, 'grm>| { p.parse(stream, cache, context).map(f) }
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
