use crate::core::cache::Allocs;
use crate::core::span::Span;
use crate::grammar::escaped_string::EscapedString;
use crate::grammar::from_action_result::parse_string;
use crate::parsable::parsed::Parsed;
use crate::parsable::Parsable;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum RuleAnnotation<'grm> {
    #[serde(borrow)]
    Error(EscapedString<'grm>),
    DisableLayout,
    EnableLayout,
    DisableRecovery,
    EnableRecovery,
}

impl<'arn, 'grm: 'arn> Parsable<'arn, 'grm> for RuleAnnotation<'grm> {
    fn from_construct(
        _span: Span,
        constructor: &'grm str,
        args: &[Parsed<'arn, 'grm>],
        _allocs: Allocs<'arn>,
        src: &'grm str,
    ) -> Self {
        match constructor {
            "Error" => RuleAnnotation::Error(parse_string(args[0], src)),
            "DisableLayout" => RuleAnnotation::DisableLayout,
            "EnableLayout" => RuleAnnotation::EnableLayout,
            "DisableRecovery" => RuleAnnotation::DisableRecovery,
            "EnableRecovery" => RuleAnnotation::EnableRecovery,
            _ => unreachable!(),
        }
    }
}
