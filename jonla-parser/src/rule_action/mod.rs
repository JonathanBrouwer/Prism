use serde::{Deserialize, Serialize};
use crate::core::adaptive::GrammarState;
use crate::core::context::RawEnv;

use crate::grammar::escaped_string::EscapedString;
use crate::grammar::grammar::Action;
use crate::rule_action::action_result::ActionResult;

pub mod apply_action;
pub mod action_result;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum RuleAction<'grm> {
    Name(&'grm str),
    InputLiteral(EscapedString<'grm>),
    Construct(&'grm str, Vec<Self>),
    Cons(Box<Self>, Box<Self>),
    Nil(),
}

impl<'grm> Action<'grm> for RuleAction<'grm> {
    fn eval_to_rule<'b>(e: &RawEnv<'b, 'grm, Self>, grammar: &'b GrammarState<'b, 'grm, Self>) -> Option<&'grm str> {
        todo!()
    }
}
