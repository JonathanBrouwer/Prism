use crate::core::input::Input;
use crate::core::input_table::InputTable;
use crate::parsable::action_result::ActionResult;
use crate::parsable::guid::Guid;
use crate::parsable::parsed::Parsed;
use crate::parser::VarMap;
use crate::parser::parsed_list::ParsedList;

impl Parsed {
    pub fn to_debug_string(&self, src: &InputTable) -> String {
        if let Some(ar) = self.try_value_ref::<ActionResult>() {
            format!(
                "{}({})",
                ar.constructor.as_str(src),
                ar.args
                    .iter()
                    .map(|e| e.to_debug_string(src))
                    .collect::<Vec<String>>()
                    .join(", ")
            )
        } else if let Some(_env) = self.try_value_ref::<VarMap>() {
            "[VARS]".to_string()
        } else if let Some(ll) = self.try_value_ref::<ParsedList>() {
            format!(
                "[{}]",
                ll.iter()
                    .map(|((), v)| v)
                    .map(|e| e.to_debug_string(src))
                    .collect::<Vec<String>>()
                    .join(", ")
            )
        } else if let Some(guid) = self.try_value_ref::<Guid>() {
            format!("Guid({})", guid.0)
        } else if let Some(input) = self.try_value_ref::<Input>() {
            format!("\'{}\'", input.to_string(src))
        } else {
            format!("Unknown value of type {}", self.name)
        }
    }
}
