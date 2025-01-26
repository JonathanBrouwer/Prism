use crate::core::input::Input;
use crate::grammar::rule_action::RuleAction;
use crate::parsable::action_result::ActionResult;
use crate::parsable::env_capture::EnvCapture;
use crate::parsable::guid::Guid;
use crate::parsable::parsed::Parsed;
use crate::parser::parsed_list::ParsedList;

impl<'arn, 'grm: 'arn> Parsed<'arn, 'grm> {
    pub fn to_debug_string(&self, src: &str) -> String {
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
        } else if let Some(env) = self.try_into_value::<EnvCapture<'arn, 'grm>>() {
            format!("Env({})", env.value.to_debug_string(src))
        } else if let Some(ll) = self.try_into_value::<ParsedList<'arn, 'grm>>() {
            format!(
                "[{}]",
                ll.into_iter()
                    .map(|e| e.to_debug_string(src))
                    .collect::<Vec<String>>()
                    .join(", ")
            )
        } else if let Some(guid) = self.try_into_value::<Guid>() {
            format!("Guid({})", guid.0)
        } else if let Some(input) = self.try_into_value::<Input>() {
            format!("\'{}\'", input.as_cow(src))
        } else if let Some(input) = self.try_into_value::<RuleAction>() {
            format!("{input:?}")
        } else {
            format!("Unknown value of type {}", self.name)
        }
    }
}
