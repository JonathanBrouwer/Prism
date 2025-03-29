use crate::core::input::Input;
use crate::core::input_table::{InputTable, META_INPUT_INDEX};
use crate::core::pos::Pos;
use crate::core::span::Span;
use crate::parsable::parsed::Parsed;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Copy, Clone)]
pub struct Identifier(Span, &'static str);

impl Identifier {
    pub fn as_str<'arn>(self, input: &InputTable<'arn>) -> &'arn str {
        if self.0.len() == usize::MAX {
            return self.1;
        }
        input.slice(self.0)
    }
}

impl<'arn> Serialize for Identifier {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0
            .unsafe_set_file(META_INPUT_INDEX)
            .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Identifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // let s: &'de str = Deserialize::deserialize(deserializer)?;
        // let s = s.to_string().leak();
        // Ok(Self(Span::new(Pos::start_of(META_INPUT_INDEX), usize::MAX), s))

        Ok(Self(Span::deserialize(deserializer)?, ""))
    }
}

pub(crate) fn parse_identifier<'arn>(r: Parsed<'arn>, src: &InputTable<'arn>) -> Identifier {
    Identifier(r.into_value::<Input>().span(), "")
}

pub(crate) fn parse_identifier_old<'arn>(r: Parsed<'arn>, src: &InputTable<'arn>) -> &'arn str {
    r.into_value::<Input>().as_str(src)
}
