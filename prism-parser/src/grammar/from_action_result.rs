use crate::core::input::Input;
use crate::core::input_table::InputTable;
use crate::grammar::escaped_string::EscapedString;
use crate::parsable::parsed::Parsed;

pub(crate) fn parse_identifier<'arn>(r: Parsed<'arn>, src: &InputTable<'arn>) -> &'arn str {
    r.into_value::<Input<'arn>>().as_str(src)
}

pub(crate) fn parse_string<'arn>(r: Parsed<'arn>, src: &InputTable<'arn>) -> EscapedString<'arn> {
    let Input::Value(span) = r.into_value::<Input<'arn>>() else {
        panic!()
    };
    EscapedString::from_escaped(src.slice(*span))
}
