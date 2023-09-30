use std::borrow::Cow;
use std::sync::Arc;

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use crate::core::adaptive::RuleId;

use crate::core::span::Span;
use crate::grammar::escaped_string::EscapedString;
use crate::grammar::grammar::{GrammarFile};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ActionResult<'grm> {
    Value(Span),
    Literal(EscapedString<'grm>),
    Construct(Span, &'grm str, Vec<ActionResult<'grm>>),
    RuleRef(RuleId),
    Grammar(Arc<GrammarFile<'grm>>),
}

impl<'grm> ActionResult<'grm> {
    pub fn get_value<'a>(&self, src: &'grm str) -> Cow<'grm, str> {
        match self {
            ActionResult::Value(span) => Cow::Borrowed(&src[*span]),
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
            ActionResult::Grammar(_) => format!("[grammar]"),
        }
    }
}
