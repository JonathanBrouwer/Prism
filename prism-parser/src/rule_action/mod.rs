use serde::{Deserialize, Serialize};

use crate::grammar::escaped_string::EscapedString;
use crate::rule_action::action_result::ActionResult;

pub mod action_result;
pub mod apply_action;
pub mod from_action_result;

#[derive(Clone, Serialize, Deserialize)]
pub enum RuleAction<'arn, 'grm> {
    Name(&'grm str),
    InputLiteral(EscapedString<'grm>),
    // TODO use more efficient structure than Vec for this
    Construct(&'grm str, Vec<Self>),
    #[serde(skip)]
    ActionResult(&'arn ActionResult<'arn, 'grm>),
}
