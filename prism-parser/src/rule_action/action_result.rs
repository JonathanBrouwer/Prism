use crate::core::adaptive::RuleId;
use crate::core::cow::Cow;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::core::span::Span;
use crate::grammar::escaped_string::EscapedString;
use crate::parser::var_map::VarMap;

#[derive(Clone, Serialize, Deserialize, Eq, PartialEq, Hash, Debug)]
pub enum ActionResult<'arn, 'grm> {
    Value(Span),
    Literal(EscapedString<'grm>),
    //TODO replace Vec by arena slice
    //TODO this can only be done after List representation is changed
    Construct(Span, &'grm str, Vec<Cow<'arn, ActionResult<'arn, 'grm>>>),
    Guid(usize),
    RuleId(RuleId),
    #[serde(skip)]
    Env(VarMap<'arn, 'grm>),
}

impl<'arn, 'grm> ActionResult<'arn, 'grm> {
    pub fn get_value(&self, src: &'grm str) -> std::borrow::Cow<'grm, str> {
        match self {
            ActionResult::Value(span) => std::borrow::Cow::Borrowed(&src[*span]),
            ActionResult::Literal(s) => s.to_cow(),
            _ => panic!("Tried to get value of non-valued action result"),
        }
    }

    pub fn to_string(&self, src: &str) -> String {
        match self {
            ActionResult::Value(span) => format!("\'{}\'", &src[*span]),
            ActionResult::Literal(lit) => format!("\'{}\'", lit),
            ActionResult::Construct(_, "List", es) => {
                format!("[{}]", es.iter().map(|e| e.to_string(src)).format(", "))
            }
            ActionResult::Construct(_, c, es) => format!(
                "{}({})",
                c,
                es.iter().map(|e| e.to_string(src)).format(", ")
            ),
            ActionResult::Guid(r) => format!("Guid({r})"),
            ActionResult::RuleId(rule) => format!("Rule({rule})"),
            ActionResult::Env(_) => "Env(...)".to_string(),
        }
    }

    pub fn void() -> Self {
        ActionResult::Construct(Span::invalid(), "#VOID#", vec![])
    }
}
