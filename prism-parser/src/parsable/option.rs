use crate::core::input::Input;
use crate::core::input_table::InputTable;
use crate::core::span::Span;
use crate::grammar::identifier::Identifier;
use crate::parsable::Parsable;
use crate::parsable::parsed::Parsed;

impl<Env> Parsable<Env> for Option<u64> {
    type EvalCtx = ();

    fn from_construct(
        _span: Span,
        constructor: Identifier,
        args: &[Parsed],

        src: &InputTable,
        _env: &mut Env,
    ) -> Self {
        match constructor.as_str(src) {
            "None" => {
                assert_eq!(args.len(), 0);
                Option::None
            }
            "Some" => {
                assert_eq!(args.len(), 1);
                Option::Some(args[0].value_ref::<Input>().as_str(src).parse().unwrap())
            }
            _ => unreachable!(),
        }
    }
}
