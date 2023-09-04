use serde::{Deserialize, Serialize};

use crate::grammar::escaped_string::EscapedString;

pub mod apply_action;
pub mod action_result;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum RuleAction<'grm> {
    Name(&'grm str),
    InputLiteral(EscapedString<'grm>),
    Construct(&'grm str, Vec<Self>),
    Cons(Box<Self>, Box<Self>),
    Nil(),
}
