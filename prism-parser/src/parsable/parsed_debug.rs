use crate::core::input::Input;
use crate::core::input_table::InputTable;
use crate::grammar::rule_action::RuleAction;
use crate::parsable::action_result::ActionResult;
use crate::parsable::guid::Guid;
use crate::parsable::parsed::Parsed;
use crate::parser::VarMap;
use crate::parser::parsed_list::ParsedList;

impl<'arn> Parsed<'arn> {
    pub fn to_debug_string(&self, src: &InputTable<'arn>) -> String {
        if let Some(ar) = self.try_into_value::<ActionResult>() {
            match ar {
                ActionResult::Construct(_, c, es) => format!(
                    "{}({})",
                    c,
                    es.iter()
                        .map(|e| e.to_debug_string(src))
                        .collect::<Vec<String>>()
                        .join(", ")
                ),
            }
        } else if let Some(_env) = self.try_into_value::<VarMap<'arn>>() {
            "[VARS]".to_string()
        } else if let Some(ll) = self.try_into_value::<ParsedList<'arn>>() {
            format!(
                "[{}]",
                ll.into_iter()
                    .map(|((), v)| v)
                    .map(|e| e.to_debug_string(src))
                    .collect::<Vec<String>>()
                    .join(", ")
            )
        } else if let Some(guid) = self.try_into_value::<Guid>() {
            format!("Guid({})", guid.0)
        } else if let Some(input) = self.try_into_value::<Input>() {
            format!("\'{}\'", input.to_string(src))
        } else {
            format!("Unknown value of type {}", self.name)
        }
    }
}
