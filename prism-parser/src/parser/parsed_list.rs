use crate::core::allocs::Allocs;
use crate::core::input_table::InputTable;
use crate::core::span::Span;
use crate::env::GenericEnv;
use crate::parsable::Parsable;
use crate::parsable::parsed::Parsed;

pub type ParsedList<'arn> = GenericEnv<'arn, (), Parsed<'arn>>;

impl<'arn, Env> Parsable<'arn, Env> for ParsedList<'arn> {
    type EvalCtx = ();

    fn from_construct(
        _span: Span,
        constructor: &'arn str,
        args: &[Parsed<'arn>],
        allocs: Allocs<'arn>,
        _src: &InputTable<'arn>,
        _env: &mut Env,
    ) -> Self {
        match constructor {
            "Cons" => {
                assert_eq!(args.len(), 2);
                args[1]
                    .into_value::<ParsedList<'arn>>()
                    .insert((), args[0], allocs)
            }
            "Nil" => ParsedList::default(),
            _ => unreachable!(),
        }
    }
}
