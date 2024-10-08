use std::env::var;
use crate::core::adaptive::RuleId;
use crate::core::span::Span;
use crate::grammar::escaped_string::EscapedString;
use crate::grammar::serde_leak::*;
use crate::parser::var_map::VarMap;
use serde::{Deserialize, Serialize};
use crate::action::action::ActionVisitor;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum ActionResult<'arn, 'grm> {
    Value(Span),
    Literal(EscapedString<'grm>),
    Construct(
        Span,
        &'grm str,
        #[serde(with = "leak_slice")] &'arn [ActionResult<'arn, 'grm>],
    ),
    Guid(usize),
    RuleId(RuleId),
    #[serde(skip)]
    Adapt(VarMap<'arn, 'grm>, &'arn ActionResult<'arn, 'grm>),
}

impl<'arn, 'grm> ActionVisitor<'arn, 'grm> for ActionResult<'arn, 'grm> {
    fn visit_value(&mut self, span: Span) -> Self {
        Self::Value(span)
    }

    fn visit_literal(&mut self, literal: EscapedString<'grm>) -> Self {
        Self::Literal(literal)
    }

    fn visit_construct(&mut self, span: Span, name: &'grm str, args: &'arn [Self]) -> Self {
        Self::Construct(span, name, args)
    }

    fn visit_guid(&mut self, guid: usize) -> Self {
        Self::Guid(guid)
    }

    fn visit_rule(&mut self, rule: RuleId) -> Self {
        Self::RuleId(rule)
    }

    fn visit_adapt(&mut self, var_map: VarMap<'arn, 'grm>, sub: &'arn Self) -> Self {
        Self::Adapt(var_map, sub)
    }
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
            ActionResult::Construct(_, "Cons" | "Nil", _) => {
                format!(
                    "[{}]",
                    self.iter_list()
                        .map(|e| e.to_string(src))
                        .collect::<Vec<String>>()
                        .join(", ")
                )
            }
            ActionResult::Construct(_, c, es) => format!(
                "{}({})",
                c,
                es.iter()
                    .map(|e| e.to_string(src))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            ActionResult::Guid(r) => format!("Guid({r})"),
            ActionResult::RuleId(rule) => format!("Rule({rule})"),
            ActionResult::Adapt(_, ar) => format!("Env({})", ar.to_string(src)),
        }
    }

    pub fn iter_list(&self) -> ARListIterator<'arn, 'grm> {
        ARListIterator(*self, None)
    }

    pub const VOID: &'static ActionResult<'static, 'static> =
        &ActionResult::Construct(Span::invalid(), "#VOID#", &[]);
}

#[derive(Clone)]
pub struct ARListIterator<'arn, 'grm: 'arn>(ActionResult<'arn, 'grm>, Option<usize>);

impl<'arn, 'grm: 'arn> Iterator for ARListIterator<'arn, 'grm> {
    type Item = &'arn ActionResult<'arn, 'grm>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0 {
            ActionResult::Construct(_, "Cons", els) => {
                assert_eq!(els.len(), 2);
                self.0 = els[1];
                self.1 = self.1.map(|v| v - 1);
                Some(&els[0])
            }
            ActionResult::Construct(_, "Nil", els) => {
                assert_eq!(els.len(), 0);
                None
            }
            _ => panic!("Invalid list: {:?}", &self.0),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let count = self.1.unwrap_or_else(|| self.clone().count());
        (count, Some(count))
    }
}

impl ExactSizeIterator for ARListIterator<'_, '_> {}
