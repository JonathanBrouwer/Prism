use serde::{Deserialize, Serialize};

use crate::grammar::escaped_string::EscapedString;
use crate::rule_action::action_result::ActionResult;
use crate::grammar::serde_leak::*;


pub mod action_result;
pub mod apply_action;
pub mod from_action_result;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum RuleAction<'arn, 'grm> {
    Name(&'grm str),
    InputLiteral(EscapedString<'grm>),
    Construct(&'grm str, #[serde(with="leak_slice")] &'arn [Self]),
    #[serde(skip)]
    ActionResult(&'arn ActionResult<'arn, 'grm>),
}
