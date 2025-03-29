use crate::core::input::Input;
use crate::core::input_table::InputTable;
use crate::parsable::parsed::Parsed;

pub(crate) fn parse_identifier<'arn>(r: Parsed<'arn>, src: &InputTable<'arn>) -> &'arn str {
    r.into_value::<Input>().as_str(src)
}
