use crate::parsable::Parsable;
use crate::parsable::parsed::Parsed;
use prism_input::input::Input;
use prism_input::input_table::InputTable;
use prism_input::span::Span;

impl<Db> Parsable<Db> for Option<u64> {
    type EvalCtx = ();

    fn from_construct(
        _span: Span,
        constructor: &str,
        args: &[Parsed],
        _env: &mut Db,
        input: &InputTable,
    ) -> Self {
        match constructor {
            "None" => {
                assert_eq!(args.len(), 0);
                Option::None
            }
            "Some" => {
                assert_eq!(args.len(), 1);
                Option::Some(args[0].value_ref::<Input>().as_str(input).parse().unwrap())
            }
            _ => unreachable!(),
        }
    }
}
