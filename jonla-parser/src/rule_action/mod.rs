use crate::core::adaptive::RuleId;
use serde::{Deserialize, Serialize};

use crate::grammar::escaped_string::EscapedString;

pub mod action_result;
pub mod apply_action;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum RuleAction<'grm> {
    Name(&'grm str),
    InputLiteral(EscapedString<'grm>),
    Construct(&'grm str, Vec<Self>),
    Cons(Box<Self>, Box<Self>),
    Nil(),
    RuleRef(RuleId),
}
