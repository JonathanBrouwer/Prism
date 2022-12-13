use crate::parser_core::span::Span;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::grammar::EscapedString;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ActionResult<'grm> {
    Value(Span),
    Literal(EscapedString<'grm>),
    Construct(&'grm str, Vec<Arc<ActionResult<'grm>>>),
    List(Vec<Arc<ActionResult<'grm>>>),
    Void(&'static str),
}

impl<'grm> ActionResult<'grm> {
    pub fn to_string(&self, src: &str) -> String {
        match self {
            ActionResult::Value(span) => format!("\'{}\'", &src[span.start..span.end]),
            ActionResult::Literal(lit) => format!("\'{}\'", lit),
            ActionResult::Construct(c, es) => format!(
                "{}({})",
                c,
                es.iter().map(|e| e.to_string(src)).format(", ")
            ),
            ActionResult::List(es) => {
                format!("[{}]", es.iter().map(|e| e.to_string(src)).format(", "))
            }
            ActionResult::Void(s) => format!("ERROR[{s}]"),
        }
    }
}
