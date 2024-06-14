use crate::core::adaptive::RuleId;
use crate::core::cow::Cow;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::core::span::Span;
use crate::grammar::escaped_string::EscapedString;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum ActionResult<'arn, 'grm> {
    Value(Span),
    Literal(EscapedString<'grm>),
    Construct(Span, &'grm str, Vec<Cow<'arn, ActionResult<'arn, 'grm>>>),
    RuleRef(RuleId),
    Guid(usize),
}

impl<'arn, 'grm> ActionResult<'arn, 'grm> {
    pub fn get_value(&self, src: &'grm str) -> std::borrow::Cow<'grm, str> {
        match self {
            ActionResult::Value(span) => std::borrow::Cow::Borrowed(&src[*span]),
            ActionResult::Literal(s) => s.to_cow(),
            _ => panic!("Tried to get value of non-valued action result"),
        }
    }

    pub fn as_rule(&self) -> RuleId {
        if let ActionResult::RuleRef(rule) = self {
            *rule
        } else {
            panic!("Tried to convert AR to rule, but it does not refer to a rule: {self:?}");
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
            ActionResult::RuleRef(r) => format!("[{r}]"),
            ActionResult::Guid(r) => format!("Guid({r})"),
        }
    }

    pub fn void() -> Self {
        ActionResult::Construct(Span::invalid(), "#VOID#", vec![])
    }
}
