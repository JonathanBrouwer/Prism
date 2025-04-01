use crate::core::input::Input;
use crate::core::span::Span;
use crate::parsable::Parsable;
use crate::parsable::parsed::Parsed;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum RuleAnnotation {
    Error(Input),
    DisableLayout,
    EnableLayout,
    DisableRecovery,
    EnableRecovery,
}

impl<Db> Parsable<Db> for RuleAnnotation {
    type EvalCtx = ();

    fn from_construct(_span: Span, constructor: &Input, args: &[Parsed], _env: &mut Db) -> Self {
        match constructor.as_str() {
            "Error" => RuleAnnotation::Error(args[0].value_ref::<Input>().parse_escaped_string()),
            "DisableLayout" => RuleAnnotation::DisableLayout,
            "EnableLayout" => RuleAnnotation::EnableLayout,
            "DisableRecovery" => RuleAnnotation::DisableRecovery,
            "EnableRecovery" => RuleAnnotation::EnableRecovery,
            _ => unreachable!(),
        }
    }
}
