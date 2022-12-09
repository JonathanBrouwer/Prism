use crate::parser::core::span::Span;
use itertools::Itertools;
use std::rc::Rc;

#[derive(Clone)]
pub enum ActionResult<'grm> {
    Value(Span),
    Literal(&'grm str),
    Construct(&'grm str, Vec<Rc<ActionResult<'grm>>>),
    List(Vec<Rc<ActionResult<'grm>>>),
    Error(&'static str),
}

impl<'grm> ActionResult<'grm> {
    pub fn to_string<'src>(&self, src: &'src str) -> String {
        match self {
            ActionResult::Value(span) => format!("\'{}\'", &src[span.start..span.end]),
            ActionResult::Literal(lit) => format!("\'{}\'", lit.to_string()),
            ActionResult::Construct(c, es) => format!(
                "{}({})",
                c,
                es.iter().map(|e| e.to_string(src)).format(", ")
            ),
            ActionResult::List(es) => {
                format!("[{}]", es.iter().map(|e| e.to_string(src)).format(", "))
            }
            ActionResult::Error(s) => format!("ERROR[{s}]"),
        }
    }
}
