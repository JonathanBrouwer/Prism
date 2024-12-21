use crate::core::adaptive::RuleId;
use crate::core::cache::Allocs;
use crate::core::parsable::{Parsable, Parsed};
use crate::core::span::Span;
use crate::grammar::escaped_string::EscapedString;
use crate::grammar::serde_leak::*;
use crate::parser::var_map::VarMap;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone)]
pub enum ActionResult<'arn, 'grm> {
    Value(Span),
    Literal(EscapedString<'grm>),
    Construct(Span, &'grm str, &'arn [Parsed<'arn, 'grm>]),
    Guid(usize),
    WithEnv(VarMap<'arn, 'grm>, &'arn ActionResult<'arn, 'grm>),
}

impl<'arn, 'grm> Parsable<'arn, 'grm> for ActionResult<'arn, 'grm> {
    fn from_span(span: Span, _text: &'arn str, _allocs: Allocs<'arn>) -> Self {
        Self::Value(span)
    }

    fn from_literal(literal: EscapedString<'grm>, _allocs: Allocs<'arn>) -> Self {
        Self::Literal(literal)
    }

    fn from_guid(guid: usize, _allocs: Allocs<'arn>) -> Self {
        Self::Guid(guid)
    }

    fn from_construct(
        span: Span,
        constructor: &'grm str,
        args: &[Parsed<'arn, 'grm>],
        allocs: Allocs<'arn>,
    ) -> Self {
        Self::Construct(span, constructor, allocs.alloc_extend(args.iter().copied()))
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
                        .map(|e| e.into_value::<ActionResult<'arn, 'grm>>().to_string(src))
                        .collect::<Vec<String>>()
                        .join(", ")
                )
            }
            ActionResult::Construct(_, c, es) => format!(
                "{}({})",
                c,
                es.iter()
                    .map(|e| e.into_value::<ActionResult<'arn, 'grm>>().to_string(src))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            ActionResult::Guid(r) => format!("Guid({r})"),
            ActionResult::WithEnv(_, ar) => format!("Env({})", ar.to_string(src)),
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
    type Item = Parsed<'arn, 'grm>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0 {
            ActionResult::Construct(_, "Cons", els) => {
                assert_eq!(els.len(), 2);
                self.0 = *els[1].into_value::<ActionResult<'arn, 'grm>>();
                self.1 = self.1.map(|v| v - 1);
                Some(els[0])
            }
            ActionResult::Construct(_, "Nil", els) => {
                assert_eq!(els.len(), 0);
                None
            }
            _ => panic!("Invalid list"),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let count = self.1.unwrap_or_else(|| self.clone().count());
        (count, Some(count))
    }
}

impl ExactSizeIterator for ARListIterator<'_, '_> {}
