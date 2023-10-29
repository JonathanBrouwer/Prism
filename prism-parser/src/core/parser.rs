use crate::core::cache::PCache;
use crate::core::context::ParserContext;
use crate::core::pos::Pos;
use crate::core::presult::PResult;
use crate::error::ParseError;

pub trait Parser<'b, 'grm, O, E: ParseError> {
    fn parse(
        &self,
        stream: Pos,
        cache: &mut PCache<'b, 'grm, E>,
        context: &ParserContext,
    ) -> PResult<O, E>;
}

pub fn map_parser<'a, 'b: 'a, 'grm: 'b, O, P, E: ParseError>(
    p: impl Parser<'b, 'grm, O, E> + 'a,
    f: &'a impl Fn(O) -> P,
) -> impl Parser<'b, 'grm, P, E> + 'a {
    move |stream: Pos, cache: &mut PCache<'b, 'grm, E>, context: &ParserContext| {
        p.parse(stream, cache, context).map(f)
    }
}

impl<
        'b,
        'grm,
        O,
        E: ParseError,
        T: Fn(Pos, &mut PCache<'b, 'grm, E>, &ParserContext) -> PResult<O, E>,
    > Parser<'b, 'grm, O, E> for T
{
    #[inline(always)]
    fn parse(
        &self,
        stream: Pos,
        cache: &mut PCache<'b, 'grm, E>,
        context: &ParserContext,
    ) -> PResult<O, E> {
        self(stream, cache, context)
    }
}