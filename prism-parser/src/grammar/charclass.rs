use crate::core::allocs::alloc_extend;
use crate::core::input::Input;
use crate::core::input_table::InputTable;
use crate::core::span::Span;
use crate::grammar::identifier::Identifier;
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

impl<Env> Parsable<Env> for CharClass {
    type EvalCtx = ();

    fn from_construct(
        _span: Span,
        constructor: Identifier,
        args: &[Parsed],

        input: &InputTable,
        _env: &mut Env,
    ) -> Self {
        assert_eq!(constructor.as_str(input), "CharClass");
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

impl<Env> Parsable<Env> for CharClassRange {
    type EvalCtx = ();

    fn from_construct(
        _span: Span,
        constructor: Identifier,
        args: &[Parsed],
        src: &InputTable,
        _env: &mut Env,
    ) -> Self {
        assert_eq!(constructor.as_str(src), "Range");
        CharClassRange(
            parse_string_char(&args[0], src),
            parse_string_char(&args[1], src),
        )
    }
}

fn parse_string_char(r: &Parsed, src: &InputTable) -> char {
    r.value_ref::<Input>().chars(src).next().unwrap()
}
