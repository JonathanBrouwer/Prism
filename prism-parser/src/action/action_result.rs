use crate::core::cache::Allocs;
use crate::core::parsable::{Parsable, Parsed};
use crate::core::span::Span;
use crate::parser::parsed_list::ParsedList;
use crate::parser::var_map::VarMap;

#[derive(Copy, Clone)]
pub enum ActionResult<'arn, 'grm> {
    Construct(Span, &'grm str, &'arn [Parsed<'arn, 'grm>]),
    WithEnv(VarMap<'arn, 'grm>, Parsed<'arn, 'grm>),
}

impl<'arn, 'grm> Parsable<'arn, 'grm> for ActionResult<'arn, 'grm> {
    fn from_construct(
        span: Span,
        constructor: &'grm str,
        args: &[Parsed<'arn, 'grm>],
        allocs: Allocs<'arn>,
    ) -> Self {
        Self::Construct(span, constructor, allocs.alloc_extend(args.iter().copied()))
    }

    fn from_construct_dyn(
        span: Span,
        constructor: &'grm str,
        args: &[Parsed<'arn, 'grm>],
        allocs: Allocs<'arn>,
    ) -> Parsed<'arn, 'grm> {
        allocs
            .alloc(Self::from_construct(span, constructor, args, allocs))
            .to_parsed()
    }
}
