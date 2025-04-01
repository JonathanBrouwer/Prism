use crate::core::input::Input;
use crate::core::span::Span;
use crate::parsable::Parsable;
use crate::parsable::parsed::Parsed;

impl<Db> Parsable<Db> for Option<u64> {
    type EvalCtx = ();

    fn from_construct(_span: Span, constructor: &Input, args: &[Parsed], _env: &mut Db) -> Self {
        match constructor.as_str() {
            "None" => {
                assert_eq!(args.len(), 0);
                Option::None
            }
            "Some" => {
                assert_eq!(args.len(), 1);
                Option::Some(args[0].value_ref::<Input>().as_str().parse().unwrap())
            }
            _ => unreachable!(),
        }
    }
}
