use crate::core::input::Input;
use crate::grammar::escaped_string::EscapedString;
use crate::parsable::action_result::ActionResult;
use crate::parsable::action_result::ActionResult::*;
use crate::parsable::parsed::Parsed;

#[macro_export]
macro_rules! result_match {
    {match $e1:expr => $p1:pat_param, $(match $es:expr => $ps:pat_param,)* create $body:expr} => {
        match $e1 {
            $p1 => {
                result_match! { $(match $es => $ps,)* create $body }
            },
            _ => None,
        }
    };
    {create $body:expr} => {
        Some($body)
    };
}

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

pub fn parse_option<'arn, 'grm, T>(
    r: &ActionResult<'arn, 'grm>,
    src: &str,
    sub: impl Fn(Parsed<'arn, 'grm>, &str) -> T,
) -> Option<T> {
    match r {
        Construct(_, "None", []) => None,
        Construct(_, "Some", b) => Some(sub(b[0], src)),
        _ => unreachable!(),
    }
}

pub fn parse_u64(r: Parsed, src: &str) -> u64 {
    r.into_value::<Input>().as_cow(src).parse().unwrap()
}
