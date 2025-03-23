use crate::core::allocs::Allocs;
use crate::core::input::Input;
use crate::core::input_table::InputTable;
use crate::core::span::Span;
use crate::parsable::parsed::Parsed;
use crate::parsable::{Parsable, ParseResult};

impl ParseResult for Option<u64> {}
impl<'arn, Env> Parsable<'arn, Env> for Option<u64> {
    type EvalCtx = ();

    fn from_construct(
        _span: Span,
        constructor: &'arn str,
        args: &[Parsed<'arn>],
        _allocs: Allocs<'arn>,
        src: &InputTable<'arn>,
        _env: &mut Env,
    ) -> Self {
        match constructor {
            "None" => {
                assert_eq!(args.len(), 0);
                Option::None
            }
            "Some" => {
                assert_eq!(args.len(), 1);
                Option::Some(args[0].into_value::<Input>().as_str(src).parse().unwrap())
            }
            _ => unreachable!(),
        }
    }
}
