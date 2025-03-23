use crate::core::input::Input;
use crate::core::input_table::InputTable;
use crate::grammar::escaped_string::EscapedString;
use crate::parsable::parsed::Parsed;

pub(crate) fn parse_identifier<'grm>(r: Parsed<'_, 'grm>, src: &InputTable<'grm>) -> &'grm str {
    r.into_value::<Input<'grm>>().as_str(src)
}

pub(crate) fn parse_string<'arn, 'grm>(
    r: Parsed<'arn, 'grm>,
    src: &InputTable<'grm>,
) -> EscapedString<'grm> {
    let Input::Value(span) = r.into_value::<Input<'grm>>() else {
        panic!()
    };
    EscapedString::from_escaped(src.slice(*span))
}
