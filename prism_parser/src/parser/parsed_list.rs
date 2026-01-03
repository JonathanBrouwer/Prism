use crate::core::input::Input;
use crate::env::GenericEnv;
use crate::parsable::Parsable;
use crate::parsable::parsed::Parsed;
use prism_input::span::Span;

pub type ParsedList = GenericEnv<(), Parsed>;

impl<Db> Parsable<Db> for ParsedList {
    type EvalCtx = ();

    fn from_construct(_span: Span, constructor: &Input, args: &[Parsed], _env: &mut Db) -> Self {
        match constructor.as_str() {
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

    fn error_fallback(_env: &mut Db, _span: Span) -> Self {
        ParsedList::default()
    }
}
