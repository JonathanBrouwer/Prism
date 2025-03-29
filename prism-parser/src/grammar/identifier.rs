use crate::core::input::Input;
use crate::core::input_table::InputTable;
use crate::core::span::Span;
use crate::parsable::parsed::Parsed;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Identifier(Span);

pub(crate) fn parse_identifier<'arn>(r: Parsed<'arn>, src: &InputTable<'arn>) -> Identifier {
    Identifier(r.into_value::<Input>().span())
}

pub(crate) fn parse_identifier_old<'arn>(r: Parsed<'arn>, src: &InputTable<'arn>) -> &'arn str {
    r.into_value::<Input>().as_str(src)
}
