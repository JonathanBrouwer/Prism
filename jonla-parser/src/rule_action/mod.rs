use serde::{Deserialize, Serialize};
use crate::core::adaptive::{RuleId};
use crate::core::context::RawEnv;

use crate::grammar::escaped_string::EscapedString;
use crate::grammar::grammar::Action;
use crate::rule_action::action_result::ActionResult;
use crate::rule_action::apply_action::apply_rawenv;

pub mod apply_action;
pub mod action_result;

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
    fn eval_to_rule<'b>(e: &RawEnv<'b, 'grm, Self>) -> Option<RuleId> {
        match apply_rawenv(e) {
            ActionResult::RuleRef(r) => Some(r),
            _ => panic!("Tried to evaluate RuleAction to rule, but it is not a rule."),
        }
    }
}
