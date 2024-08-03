use crate::core::adaptive::RuleId;
use itertools::Itertools;
use serde::Serialize;

use crate::core::span::Span;
use crate::grammar::escaped_string::EscapedString;
use crate::parser::var_map::VarMap;

#[derive(Copy, Clone, Serialize, Eq, PartialEq, Hash, Debug)]
pub enum ActionResult<'arn, 'grm> {
    Value(Span),
    Literal(EscapedString<'grm>),
    Construct(Span, &'grm str, &'arn [ActionResult<'arn, 'grm>]),
    Guid(usize),
    RuleId(RuleId),
    #[serde(skip)]
    WithEnv(VarMap<'arn, 'grm>, &'arn ActionResult<'arn, 'grm>),
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
                format!("[{}]", self.iter_list().map(|e| e.to_string(src)).format(", "))
            }
            ActionResult::Construct(_, c, es) => format!(
                "{}({})",
                c,
                es.iter().map(|e| e.to_string(src)).format(", ")
            ),
            ActionResult::Guid(r) => format!("Guid({r})"),
            ActionResult::RuleId(rule) => format!("Rule({rule})"),
            ActionResult::WithEnv(_, ar) => format!("Env({})", ar.to_string(src)),
        }
    }

    pub fn iter_list(&self) -> impl Iterator<Item=&'arn Self> + 'arn {
        ARListIterator(*self)
    }

    pub const VOID: &'static ActionResult<'static, 'static> = &ActionResult::Construct(Span::invalid(), "#VOID#", &[]);
}

pub struct ARListIterator<'arn, 'grm: 'arn>(ActionResult<'arn, 'grm>);

impl<'arn, 'grm: 'arn> Iterator for ARListIterator<'arn, 'grm> {
    type Item = &'arn ActionResult<'arn, 'grm>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0 {
            ActionResult::Construct(_, "Cons", els) => {
                assert_eq!(els.len(), 2);
                self.0 = els[1];
                Some(&els[0])
            }
            ActionResult::Construct(_, "Nil", els) => {
                assert_eq!(els.len(), 0);
                None
            }
            _ => panic!("Invalid list: {:?}", &self.0),
        }
    }
}
