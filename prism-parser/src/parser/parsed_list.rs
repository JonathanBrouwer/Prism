use crate::core::input_table::InputTable;
use crate::core::span::Span;
use crate::env::GenericEnv;
use crate::grammar::identifier::Identifier;
use crate::parsable::Parsable;
use crate::parsable::parsed::Parsed;

pub type ParsedList = GenericEnv<(), Parsed>;

impl<Env> Parsable<Env> for ParsedList {
    type EvalCtx = ();

    fn from_construct(
        _span: Span,
        constructor: Identifier,
        args: &[Parsed],

        src: &InputTable,
        _env: &mut Env,
    ) -> Self {
        match constructor.as_str(src) {
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
