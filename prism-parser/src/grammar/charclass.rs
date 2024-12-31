use crate::core::cache::Allocs;
use crate::core::input::Input;
use crate::core::span::Span;
use crate::grammar::annotated_rule_expr::AnnotatedRuleExpr;
use crate::grammar::serde_leak::*;
use crate::parsable::parsed::Parsed;
use crate::parsable::{Parsable2, ParseResult};
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

impl<'arn, 'grm: 'arn> ParseResult<'arn, 'grm> for CharClass<'arn> {}
impl<'arn, 'grm: 'arn, Env> Parsable2<'arn, 'grm, Env> for CharClass<'arn> {
    fn from_construct(
        _span: Span,
        constructor: &'grm str,
        _args: &[Parsed<'arn, 'grm>],
        _allocs: Allocs<'arn>,
        _src: &'grm str,
        _env: &mut Env,
    ) -> Self {
        assert_eq!(constructor, "CharClass");
        CharClass {
            neg: _args[0]
                .into_value::<ParsedList>()
                .into_iter()
                .next()
                .is_some(),
            ranges: _allocs.alloc_extend(
                _args[1]
                    .into_value::<ParsedList>()
                    .into_iter()
                    .map(|p| *p.into_value::<CharClassRange>()),
            ),
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct CharClassRange(char, char);

impl<'arn, 'grm: 'arn> ParseResult<'arn, 'grm> for CharClassRange {}
impl<'arn, 'grm: 'arn, Env> Parsable2<'arn, 'grm, Env> for CharClassRange {
    fn from_construct(
        _span: Span,
        constructor: &'grm str,
        _args: &[Parsed<'arn, 'grm>],
        _allocs: Allocs<'arn>,
        _src: &'grm str,
        _env: &mut Env,
    ) -> Self {
        assert_eq!(constructor, "Range");
        CharClassRange(
            parse_string_char(_args[0], _src),
            parse_string_char(_args[1], _src),
        )
    }
}

fn parse_string_char(r: Parsed, src: &str) -> char {
    r.into_value::<Input>().as_cow(src).chars().next().unwrap()
}
