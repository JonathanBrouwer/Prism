use crate::core::allocs::Allocs;
use crate::core::input_table::InputTable;
use crate::core::span::Span;
use crate::grammar::escaped_string::EscapedString;
use crate::grammar::from_action_result::parse_string;
use crate::parsable::parsed::Parsed;
use crate::parsable::{Parsable, ParseResult};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Debug)]
pub enum RuleAnnotation<'arn> {
    #[serde(borrow)]
    Error(EscapedString<'arn>),
    DisableLayout,
    EnableLayout,
    DisableRecovery,
    EnableRecovery,
}

impl ParseResult for RuleAnnotation<'_> {}
impl<'arn, Env> Parsable<'arn, Env> for RuleAnnotation<'arn> {
    type EvalCtx = ();

    fn from_construct(
        _span: Span,
        constructor: &'arn str,
        args: &[Parsed<'arn>],
        _allocs: Allocs<'arn>,
        src: &InputTable<'arn>,
        _env: &mut Env,
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
