use crate::action::action_result::ActionResult;
use crate::core::input::Input;
use crate::core::parsable::{Guid, Parsed};

impl<'arn, 'grm: 'arn> Parsed<'arn, 'grm> {
    pub fn to_string(&self, src: &str) -> String {
        if let Some(ar) = self.try_into_value::<ActionResult>() {
            match ar {
                ActionResult::Construct(_, "Cons" | "Nil", _) => {
                    format!(
                        "[{}]",
                        ar.iter_list()
                            .map(|e| e.to_string(src))
                            .collect::<Vec<String>>()
                            .join(", ")
                    )
                }
                ActionResult::Construct(_, c, es) => format!(
                    "{}({})",
                    c,
                    es.iter()
                        .map(|e| e.to_string(src))
                        .collect::<Vec<String>>()
                        .join(", ")
                ),
                ActionResult::WithEnv(_, ar) => format!("Env({})", ar.to_string(src)),
            }
        } else if let Some(guid) = self.try_into_value::<Guid>() {
            format!("Guid({})", guid.0)
        } else if let Some(input) = self.try_into_value::<Input>() {
            format!("\'{}\'", input.as_cow(src))
        } else {
            panic!("Could not debug print unknown parsed")
        }
    }
}
