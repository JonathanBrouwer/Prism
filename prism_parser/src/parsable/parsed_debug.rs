use crate::core::input::Input;
use crate::parsable::action_result::ActionResult;
use crate::parsable::guid::Guid;
use crate::parsable::parsed::Parsed;
use crate::parser::VarMap;
use crate::parser::parsed_list::ParsedList;
use std::fmt::{Debug, Formatter};

impl Debug for Parsed {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(ar) = self.try_value_ref::<ActionResult>() {
            write!(
                f,
                "{}({})",
                ar.constructor.as_str(),
                ar.args
                    .iter()
                    .map(|e| format!("{e:?}"))
                    .collect::<Vec<String>>()
                    .join(", ")
            )
        } else if let Some(_env) = self.try_value_ref::<VarMap>() {
            write!(f, "[VARS]")
        } else if let Some(ll) = self.try_value_ref::<ParsedList>() {
            write!(
                f,
                "[{}]",
                ll.iter()
                    .map(|((), v)| v)
                    .map(|e| format!("{e:?}"))
                    .collect::<Vec<String>>()
                    .join(", ")
            )
        } else if let Some(guid) = self.try_value_ref::<Guid>() {
            write!(f, "Guid({})", guid.0)
        } else if let Some(input) = self.try_value_ref::<Input>() {
            write!(f, "\'{input}\'")
        } else {
            write!(f, "Unknown value of type {}", self.name)
        }
    }
}
