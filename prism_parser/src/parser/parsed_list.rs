use crate::env::GenericEnv;
use crate::parsable::Parsable;
use crate::parsable::parsed::Parsed;
use prism_input::input_table::InputTable;
use prism_input::span::Span;

pub type ParsedList = GenericEnv<(), Parsed>;

impl<Db> Parsable<Db> for ParsedList {
    type EvalCtx = ();

    fn from_construct(
        _span: Span,
        constructor: &str,
        args: &[Parsed],
        _env: &mut Db,
        _input: &InputTable,
    ) -> Self {
        match constructor {
            "Cons" => {
                assert_eq!(args.len(), 2);
                args[1]
                    .value_ref::<ParsedList>()
                    .insert((), args[0].clone())
            }
            "Nil" => ParsedList::default(),
            _ => unreachable!(),
        }
    }
}
