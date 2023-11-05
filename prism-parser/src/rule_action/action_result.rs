use crate::core::adaptive::RuleId;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use crate::core::cow::Cow;

use crate::core::span::Span;
use crate::grammar::escaped_string::EscapedString;

//TODO should not be clone as well (after cow is fixed)
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum ActionResult<'b, 'grm> {
    Value(Span),
    Literal(EscapedString<'grm>),
    Construct(Span, &'grm str, Vec<Cow<'b, ActionResult<'b, 'grm>>>),
    RuleRef(RuleId),
}

impl<'b, 'grm> ActionResult<'b, 'grm> {
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
            ActionResult::RuleRef(r) => format!("[{}]", r),
        }
    }

    pub fn void() -> Self {
        ActionResult::Construct(Span::invalid(), "#VOID#", vec![])
    }
}
