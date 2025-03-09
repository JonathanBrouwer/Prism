use crate::core::allocs::Allocs;
use crate::core::input::Input;
use crate::core::span::Span;
use crate::parsable::parsed::Parsed;
use crate::parsable::{Parsable, ParseResult};

impl<'arn, 'grm: 'arn> ParseResult<'arn, 'grm> for Option<u64> {}
impl<'arn, 'grm: 'arn, Env> Parsable<'arn, 'grm, Env> for Option<u64> {
    type EvalCtx = ();

    fn from_construct(
        _span: Span,
        constructor: &'grm str,
        _args: &[Parsed<'arn, 'grm>],
        _allocs: Allocs<'arn>,
        _src: &'grm str,
        _env: &mut Env,
    ) -> Self {
        match constructor {
            "None" => {
                assert_eq!(_args.len(), 0);
                Option::None
            }
            "Some" => {
                assert_eq!(_args.len(), 1);
                Option::Some(_args[0].into_value::<Input>().as_str(_src).parse().unwrap())
            }
            _ => unreachable!(),
        }
    }
}
