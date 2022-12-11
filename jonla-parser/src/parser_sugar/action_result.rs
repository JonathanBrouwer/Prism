use crate::parser_core::span::Span;
use itertools::Itertools;
use std::rc::Rc;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ActionResult<'grm> {
    Value(Span),
    Literal(&'grm str),
    Construct(&'grm str, Vec<Rc<ActionResult<'grm>>>),
    List(Vec<Rc<ActionResult<'grm>>>),
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
