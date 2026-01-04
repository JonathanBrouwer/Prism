use crate::core::allocs::alloc_extend;
use crate::parsable::Parsable;
use crate::parsable::parsed::Parsed;
use crate::parser::parsed_list::ParsedList;
use prism_input::input::Input;
use prism_input::input_table::InputTable;
use prism_input::span::Span;
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

    fn from_construct(
        _span: Span,
        constructor: &str,
        args: &[Parsed],
        _env: &mut Db,
        _input: &InputTable,
    ) -> Self {
        assert_eq!(constructor, "CharClass");
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

    fn from_construct(
        _span: Span,
        constructor: &str,
        args: &[Parsed],
        _env: &mut Db,
        input: &InputTable,
    ) -> Self {
        assert_eq!(constructor, "Range");
        CharClassRange(
            parse_string_char(&args[0], input),
            parse_string_char(&args[1], input),
        )
    }
}

fn parse_string_char(r: &Parsed, input: &InputTable) -> char {
    r.value_ref::<Input>().as_str(input).chars().next().unwrap()
}
