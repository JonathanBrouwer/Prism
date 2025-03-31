use crate::core::input::Input;
use crate::core::input_table::InputTable;
use crate::core::span::Span;
use crate::grammar::identifier::Identifier;
use crate::parsable::Parsable;
use crate::parsable::parsed::Parsed;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub enum RuleAnnotation {
    Error(Input),
    DisableLayout,
    EnableLayout,
    DisableRecovery,
    EnableRecovery,
}

impl<Env> Parsable<Env> for RuleAnnotation {
    type EvalCtx = ();

    fn from_construct(
        _span: Span,
        constructor: Identifier,
        args: &[Parsed],

        src: &InputTable,
        _env: &mut Env,
    ) -> Self {
        match constructor.as_str(src) {
            "Error" => RuleAnnotation::Error(args[0].value_ref::<Input>().parse_escaped_string()),
            "DisableLayout" => RuleAnnotation::DisableLayout,
            "EnableLayout" => RuleAnnotation::EnableLayout,
            "DisableRecovery" => RuleAnnotation::DisableRecovery,
            "EnableRecovery" => RuleAnnotation::EnableRecovery,
            _ => unreachable!(),
        }
    }
}
