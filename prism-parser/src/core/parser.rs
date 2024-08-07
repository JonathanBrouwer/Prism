use crate::core::context::ParserContext;
use crate::core::pos::Pos;
use crate::core::presult::PResult;
use crate::core::state::PState;
use crate::error::ParseError;

pub trait Parser<'arn, 'grm: 'arn, O, E: ParseError> {
    fn parse(
        &self,
        pos: Pos,
        state: &mut PState<'arn, 'grm, E>,
        context: ParserContext,
    ) -> PResult<O, E>;
}

pub fn map_parser<'a, 'arn: 'a, 'grm: 'arn, O, P, E: ParseError>(
    p: impl Parser<'arn, 'grm, O, E> + 'a,
    f: &'a impl Fn(O) -> P,
) -> impl Parser<'arn, 'grm, P, E> + 'a {
    move |pos: Pos, state: &mut PState<'arn, 'grm, E>, context: ParserContext| {
        p.parse(pos, state, context).map(f)
    }
}

impl<
        'arn,
        'grm: 'arn,
        O,
        E: ParseError,
        T: Fn(Pos, &mut PState<'arn, 'grm, E>, ParserContext) -> PResult<O, E>,
    > Parser<'arn, 'grm, O, E> for T
{
    #[inline(always)]
    fn parse(
        &self,
        pos: Pos,
        state: &mut PState<'arn, 'grm, E>,
        context: ParserContext,
    ) -> PResult<O, E> {
        self(pos, state, context)
    }
}
