use crate::core::allocs::Allocs;
use crate::core::input_table::InputTable;
use crate::core::span::Span;
use crate::env::GenericEnv;
use crate::parsable::Parsable;
use crate::parsable::parsed::Parsed;

pub type ParsedList<'arn, 'grm> = GenericEnv<'arn, (), Parsed<'arn, 'grm>>;

impl<'arn, 'grm, Env> Parsable<'arn, 'grm, Env> for ParsedList<'arn, 'grm> {
    type EvalCtx = ();

    fn from_construct(
        _span: Span,
        constructor: &'grm str,
        _args: &[Parsed<'arn, 'grm>],
        _allocs: Allocs<'arn>,
        _src: &InputTable<'grm>,
        _env: &mut Env,
    ) -> Self {
        match constructor {
            "Cons" => {
                assert_eq!(_args.len(), 2);
                _args[1]
                    .into_value::<ParsedList<'arn, 'grm>>()
                    .insert((), _args[0], _allocs)
            }
            "Nil" => ParsedList::default(),
            _ => unreachable!(),
        }
    }
}
