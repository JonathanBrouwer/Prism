use crate::core::cache::PCache;
use crate::core::context::ParserContext;
use crate::core::pos::Pos;
use crate::core::presult::PResult;
use crate::error::ParseError;
use crate::grammar::grammar::Action;

pub trait Parser<'b, 'grm, O, E: ParseError, A: Action<'grm>> {
    fn parse(
        &self,
        stream: Pos,
        cache: &mut PCache<'b, 'grm, E, A>,
        context: &ParserContext,
    ) -> PResult<O, E>;
}

pub fn map_parser<'a, 'b: 'a, 'grm: 'b, O, P, E: ParseError, A: Action<'grm>>(
    p: impl Parser<'b, 'grm, O, E, A> + 'a,
    f: &'a impl Fn(O) -> P,
) -> impl Parser<'b, 'grm, P, E, A> + 'a {
    move |stream: Pos, cache: &mut PCache<'b, 'grm, E, A>, context: &ParserContext| {
        p.parse(stream, cache, context).map(f)
    }
}

impl<
        'b,
        'grm,
        O,
        E: ParseError,
        A: Action<'grm>,
        T: Fn(Pos, &mut PCache<'b, 'grm, E, A>, &ParserContext) -> PResult<O, E>,
    > Parser<'b, 'grm, O, E, A> for T
{
    #[inline(always)]
    fn parse(
        &self,
        stream: Pos,
        cache: &mut PCache<'b, 'grm, E, A>,
        context: &ParserContext,
    ) -> PResult<O, E> {
        self(stream, cache, context)
    }
}
