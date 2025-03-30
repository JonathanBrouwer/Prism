use crate::core::allocs::Allocs;
use crate::core::input::Input;
use crate::core::input_table::InputTable;
use crate::core::span::Span;
use crate::grammar::identifier::Identifier;
use crate::parsable::parsed::Parsed;
use crate::parsable::{Parsable, ParseResult};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub enum RuleAnnotation {
    Error(Input),
    DisableLayout,
    EnableLayout,
    DisableRecovery,
    EnableRecovery,
}

impl ParseResult for RuleAnnotation {}
impl<'arn, Env> Parsable<'arn, Env> for RuleAnnotation {
    type EvalCtx = ();

    fn from_construct(
        _span: Span,
        constructor: Identifier,
        args: &[Parsed<'arn>],
        _allocs: Allocs<'arn>,
        src: &InputTable<'arn>,
        _env: &mut Env,
    ) -> Self {
        match constructor.as_str(src) {
            "Error" => RuleAnnotation::Error(args[0].into_value::<Input>().parse_escaped_string()),
            "DisableLayout" => RuleAnnotation::DisableLayout,
            "EnableLayout" => RuleAnnotation::EnableLayout,
            "DisableRecovery" => RuleAnnotation::DisableRecovery,
            "EnableRecovery" => RuleAnnotation::EnableRecovery,
            _ => unreachable!(),
        }
    }
}
