use crate::core::adaptive::RuleId;
use serde::{Deserialize, Serialize};
use crate::grammar::Action;

use crate::grammar::escaped_string::EscapedString;
use crate::rule_action::action_result::ActionResult;
use crate::rule_action::from_action_result::parse_rule_action;

pub mod action_result;
pub mod apply_action;
pub mod from_action_result;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum RuleAction<'grm> {
    Name(&'grm str),
    InputLiteral(EscapedString<'grm>),
    Construct(&'grm str, Vec<Self>),
    Cons(Box<Self>, Box<Self>),
    Nil(),
    RuleRef(RuleId),
}

impl<'grm> Action<'grm> for RuleAction<'grm> {
    fn parse_action(r: &ActionResult<'grm>, src: &'grm str) -> Option<Self> {
        parse_rule_action(r, src)
    }
}