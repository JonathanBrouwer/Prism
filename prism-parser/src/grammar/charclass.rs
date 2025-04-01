use crate::core::allocs::alloc_extend;
use crate::core::input::Input;
use crate::core::span::Span;
use crate::parsable::Parsable;
use crate::parsable::parsed::Parsed;
use crate::parser::parsed_list::ParsedList;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct CharClass {
    pub neg: bool,
    pub ranges: Arc<[Arc<CharClassRange>]>,
}

impl CharClass {
    pub fn contains(&self, c: char) -> bool {
        self.ranges.iter().any(|range| range.0 <= c && c <= range.1) ^ self.neg
    }
}

impl<Db> Parsable<Db> for CharClass {
    type EvalCtx = ();

    fn from_construct(_span: Span, constructor: &Input, args: &[Parsed], _env: &mut Db) -> Self {
        assert_eq!(constructor.as_str(), "CharClass");
        CharClass {
            neg: args[0].value_ref::<ParsedList>().iter().next().is_some(),
            ranges: alloc_extend(
                args[1]
                    .value_ref::<ParsedList>()
                    .iter()
                    .map(|((), v)| v)
                    .map(|p| p.value_cloned::<CharClassRange>()),
            ),
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct CharClassRange(char, char);

impl<Db> Parsable<Db> for CharClassRange {
    type EvalCtx = ();

    fn from_construct(_span: Span, constructor: &Input, args: &[Parsed], _env: &mut Db) -> Self {
        assert_eq!(constructor.as_str(), "Range");
        CharClassRange(parse_string_char(&args[0]), parse_string_char(&args[1]))
    }
}

fn parse_string_char(r: &Parsed) -> char {
    r.value_ref::<Input>().as_str().chars().next().unwrap()
}
