use crate::core::adaptive::RuleId;
use itertools::Itertools;
#[cfg(feature = "serde_leaking_action_result")]
use serde::{Deserialize, Serialize};
use crate::core::span::Span;
use crate::grammar::escaped_string::EscapedString;
use crate::parser::var_map::VarMap;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[cfg_attr(feature = "serde_leaking_action_result", derive(Serialize, Deserialize))]
pub enum ActionResult<'arn, 'grm> {
    Value(Span),
    Literal(EscapedString<'grm>),
    Construct(Span, &'grm str, #[cfg_attr(feature = "serde_leaking_action_result", serde(with="leak_slice"))] &'arn [ActionResult<'arn, 'grm>]),
    Guid(usize),
    RuleId(RuleId),
    #[cfg_attr(feature = "serde_leaking_action_result", serde(skip))]
    WithEnv(VarMap<'arn, 'grm>, &'arn ActionResult<'arn, 'grm>),
}

#[cfg(feature = "serde_leaking_action_result")]
pub mod leak_slice {
    use std::fmt;
    use std::fmt::Formatter;
    use std::marker::PhantomData;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use serde::de::{SeqAccess, Visitor};
    use serde::ser::SerializeSeq;

    pub fn serialize<S: Serializer, T: Serialize>(xs: &[T], s: S) -> Result<S::Ok, S::Error> {
        let mut seq = s.serialize_seq(Some(xs.len()))?;
        for x in xs {
            seq.serialize_element(x)?;
        }
        seq.end()
    }

    pub fn deserialize<'de, D: Deserializer<'de>, T: Deserialize<'de>>(deserializer: D) -> Result<&'de [T], D::Error> {
        struct VecVisitor<T> {
            marker: PhantomData<T>,
        }

        impl<'de, T> Visitor<'de> for VecVisitor<T>
        where
            T: Deserialize<'de>,
        {
            type Value = Vec<T>;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("a sequence")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut values = Vec::<T>::with_capacity(seq.size_hint().unwrap_or(0));
                while let Some(value) = seq.next_element()? {
                    values.push(value);
                }
                Ok(values)
            }
        }

        let visitor = VecVisitor {
            marker: PhantomData,
        };
        let vec: Vec<T> = deserializer.deserialize_seq(visitor)?;
        Ok(vec.leak())
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
                    self.iter_list().map(|e| e.to_string(src)).format(", ")
                )
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

    pub fn iter_list(&self) -> impl Iterator<Item = &'arn Self> + 'arn {
        ARListIterator(*self)
    }

    pub const VOID: &'static ActionResult<'static, 'static> =
        &ActionResult::Construct(Span::invalid(), "#VOID#", &[]);
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
