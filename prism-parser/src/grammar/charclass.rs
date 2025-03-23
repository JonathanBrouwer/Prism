use crate::core::allocs::Allocs;
use crate::core::input::Input;
use crate::core::input_table::InputTable;
use crate::core::span::Span;
use crate::grammar::serde_leak::*;
use crate::parsable::parsed::Parsed;
use crate::parsable::{Parsable, ParseResult};
use crate::parser::parsed_list::ParsedList;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct CharClass<'arn> {
    pub neg: bool,
    #[serde(borrow, with = "leak_slice")]
    pub ranges: &'arn [CharClassRange],
}

impl CharClass<'_> {
    pub fn contains(&self, c: char) -> bool {
        self.ranges.iter().any(|range| range.0 <= c && c <= range.1) ^ self.neg
    }
}

impl<'arn> ParseResult<'arn> for CharClass<'arn> {}
impl<'arn, Env> Parsable<'arn, Env> for CharClass<'arn> {
    type EvalCtx = ();

    fn from_construct(
        _span: Span,
        constructor: &'arn str,
        args: &[Parsed<'arn>],
        allocs: Allocs<'arn>,
        _src: &InputTable<'arn>,
        _env: &mut Env,
    ) -> Self {
        assert_eq!(constructor, "CharClass");
        CharClass {
            neg: args[0]
                .into_value::<ParsedList>()
                .into_iter()
                .next()
                .is_some(),
            ranges: allocs.alloc_extend(
                args[1]
                    .into_value::<ParsedList>()
                    .into_iter()
                    .map(|((), v)| v)
                    .map(|p| *p.into_value::<CharClassRange>()),
            ),
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct CharClassRange(char, char);

impl<'arn> ParseResult<'arn> for CharClassRange {}
impl<'arn, Env> Parsable<'arn, Env> for CharClassRange {
    type EvalCtx = ();

    fn from_construct(
        _span: Span,
        constructor: &'arn str,
        args: &[Parsed<'arn>],
        _allocs: Allocs<'arn>,
        src: &InputTable<'arn>,
        _env: &mut Env,
    ) -> Self {
        assert_eq!(constructor, "Range");
        CharClassRange(
            parse_string_char(args[0], src),
            parse_string_char(args[1], src),
        )
    }
}

fn parse_string_char<'arn>(r: Parsed<'arn>, src: &InputTable<'arn>) -> char {
    r.into_value::<Input>().as_cow(src).chars().next().unwrap()
}
