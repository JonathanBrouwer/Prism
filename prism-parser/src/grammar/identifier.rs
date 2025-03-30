use crate::META_GRAMMAR_STR;
use crate::core::input::Input;
use crate::core::input_table::{InputTable, META_INPUT_INDEX};
use crate::core::pos::Pos;
use crate::core::span::Span;
use crate::grammar::serde_leak::leak;
use crate::parsable::parsed::Parsed;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Copy, Clone, Debug)]
pub enum Identifier {
    FromSource(Span),
    Const(&'static str),
}

impl Identifier {
    pub fn as_str<'arn>(self, input: &InputTable<'arn>) -> &'arn str {
        match self {
            Identifier::FromSource(span) => input.slice(span),
            Identifier::Const(c) => c,
        }
    }

    pub fn from_const(s: &'static str) -> Self {
        Self::Const(s)
    }
}

impl<'arn> Serialize for Identifier {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match self {
            Identifier::FromSource(v) => {
                &META_GRAMMAR_STR
                    [v.start_pos().idx_in_file()..v.start_pos().idx_in_file() + v.len()]
            }
            Identifier::Const(c) => c,
        };
        s.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Identifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &'de str = Deserialize::deserialize(deserializer)?;
        Ok(Self::Const(s.to_string().leak()))
    }
}

pub(crate) fn parse_identifier<'arn>(r: Parsed<'arn>) -> Identifier {
    Identifier::FromSource(r.into_value::<Input>().span())
}
