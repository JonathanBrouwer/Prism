use crate::core::input::Input;
use crate::grammar::escaped_string::EscapedString;
use crate::parsable::action_result::ActionResult;
use crate::parsable::action_result::ActionResult::*;
use crate::parsable::parsed::Parsed;

pub(crate) fn parse_identifier<'grm>(r: Parsed<'_, 'grm>, src: &'grm str) -> &'grm str {
    r.into_value::<Input<'grm>>().as_str(src)
}

pub(crate) fn parse_string<'arn, 'grm>(
    r: Parsed<'arn, 'grm>,
    src: &'grm str,
) -> EscapedString<'grm> {
    let Input::Value(span) = r.into_value::<Input<'grm>>() else {
        panic!()
    };
    EscapedString::from_escaped(&src[*span])
}

pub fn parse_u64(r: Parsed, src: &str) -> u64 {
    r.into_value::<Input>().as_cow(src).parse().unwrap()
}
