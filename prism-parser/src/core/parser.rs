use crate::core::cache::PCache;
use crate::core::context::ParserContext;
use crate::core::pos::Pos;
use crate::core::presult::PResult;
use crate::error::ParseError;

pub trait Parser<'arn, 'grm, O, E: ParseError> {
    fn parse(
        &self,
        stream: Pos,
        cache: &mut PCache<'arn, 'grm, E>,
        context: &ParserContext,
    ) -> PResult<O, E>;
}

pub fn map_parser<'a, 'arn: 'a, 'grm: 'arn, O, P, E: ParseError>(
    p: impl Parser<'arn, 'grm, O, E> + 'a,
    f: &'a impl Fn(O) -> P,
) -> impl Parser<'arn, 'grm, P, E> + 'a {
    move |stream: Pos, cache: &mut PCache<'arn, 'grm, E>, context: &ParserContext| {
        p.parse(stream, cache, context).map(f)
    }
}

impl<
        'arn,
        'grm,
        O,
        E: ParseError,
        T: Fn(Pos, &mut PCache<'arn, 'grm, E>, &ParserContext) -> PResult<O, E>,
    > Parser<'arn, 'grm, O, E> for T
{
    #[inline(always)]
    fn parse(
        &self,
        stream: Pos,
        cache: &mut PCache<'arn, 'grm, E>,
        context: &ParserContext,
    ) -> PResult<O, E> {
        self(stream, cache, context)
    }
}
