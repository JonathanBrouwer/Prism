use crate::grammar::action_result::ActionResult;
use crate::grammar::escaped_string::EscapedString;
use crate::grammar::serde_leak::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum RuleAction<'arn, 'grm> {
    Name(&'grm str),
    InputLiteral(EscapedString<'grm>),
    Construct(&'grm str, #[serde(with = "leak_slice")] &'arn [Self]),
    #[serde(skip)]
    ActionResult(&'arn ActionResult<'arn, 'grm>),
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub enum RuleActionType<'arn, 'grm> {
    Name(&'grm str),
    Input,
    Rule,
    List(#[serde(with="leak")] &'arn Self),
}