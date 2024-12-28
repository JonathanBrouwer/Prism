use crate::core::cache::Allocs;
use crate::core::input::Input;
use crate::core::span::Span;
use crate::parsable::parsed::Parsed;
use crate::parsable::Parsable;
use std::any::type_name;
use std::str::FromStr;

impl<'arn, 'grm: 'arn> Parsable<'arn, 'grm> for Option<u64> {
    fn from_construct(
        _span: Span,
        constructor: &'grm str,
        args: &[Parsed<'arn, 'grm>],
        _allocs: Allocs<'arn>,
        src: &'grm str,
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
