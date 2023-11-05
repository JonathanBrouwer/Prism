use crate::core::cow::Cow;
use crate::core::adaptive::RuleId;
use serde::{Deserialize, Serialize};

use crate::grammar::escaped_string::EscapedString;
use crate::rule_action::action_result::ActionResult;

pub mod action_result;
pub mod apply_action;
pub mod from_action_result;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum RuleAction<'b, 'grm> {
    Name(&'grm str),
    InputLiteral(EscapedString<'grm>),
    // TODO use more efficient structure than Vec for this
    Construct(&'grm str, Vec<Self>),
    Cons(Box<Self>, Box<Self>),
    Nil(),
    RuleRef(RuleId),
    #[serde(skip)]
    ActionResult(&'b ActionResult<'b, 'grm>),
}
