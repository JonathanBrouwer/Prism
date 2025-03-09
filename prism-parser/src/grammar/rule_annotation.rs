use crate::core::allocs::Allocs;
use crate::core::span::Span;
use crate::grammar::escaped_string::EscapedString;
use crate::grammar::from_action_result::parse_string;
use crate::parsable::parsed::Parsed;
use crate::parsable::{Parsable, ParseResult};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Debug)]
pub enum RuleAnnotation<'grm> {
    #[serde(borrow)]
    Error(EscapedString<'grm>),
    DisableLayout,
    EnableLayout,
    DisableRecovery,
    EnableRecovery,
}

impl<'arn, 'grm: 'arn> ParseResult<'arn, 'grm> for RuleAnnotation<'grm> {}
impl<'arn, 'grm: 'arn, Env> Parsable<'arn, 'grm, Env> for RuleAnnotation<'grm> {
    type EvalCtx = ();

    fn from_construct(
        _span: Span,
        constructor: &'grm str,
        _args: &[Parsed<'arn, 'grm>],
        _allocs: Allocs<'arn>,
        _src: &'grm str,
        _env: &mut Env,
    ) -> Self {
        match constructor {
            "Error" => RuleAnnotation::Error(parse_string(_args[0], _src)),
            "DisableLayout" => RuleAnnotation::DisableLayout,
            "EnableLayout" => RuleAnnotation::EnableLayout,
            "DisableRecovery" => RuleAnnotation::DisableRecovery,
            "EnableRecovery" => RuleAnnotation::EnableRecovery,
            _ => unreachable!(),
        }
    }
}
