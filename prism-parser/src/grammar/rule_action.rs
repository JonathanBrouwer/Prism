use crate::core::cache::Allocs;
use crate::core::span::Span;
use crate::grammar::escaped_string::EscapedString;
use crate::grammar::from_action_result::{parse_identifier, parse_string};
use crate::grammar::serde_leak::*;
use crate::parsable::parsed::Parsed;
use crate::parsable::{Parsable2, ParseResult};
use crate::parser::parsed_list::ParsedList;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum RuleAction<'arn, 'grm> {
    Name(&'grm str),
    InputLiteral(EscapedString<'grm>),
    Construct(
        &'grm str,
        &'grm str,
        #[serde(with = "leak_slice")] &'arn [Self],
    ),
    #[serde(skip)]
    Value(Parsed<'arn, 'grm>),
}

impl<'arn, 'grm: 'arn> ParseResult<'arn, 'grm> for RuleAction<'arn, 'grm> {}
impl<'arn, 'grm: 'arn, Env: Copy> Parsable2<'arn, 'grm, Env> for RuleAction<'arn, 'grm> {
    fn from_construct(
        _span: Span,
        constructor: &'grm str,
        args: &[Parsed<'arn, 'grm>],
        allocs: Allocs<'arn>,
        src: &'grm str,
    ) -> Self {
        match constructor {
            "Construct" => RuleAction::Construct(
                parse_identifier(args[0], src),
                parse_identifier(args[1], src),
                allocs.alloc_extend(
                    args[2]
                        .into_value::<ParsedList>()
                        .into_iter()
                        .map(|sub| *sub.into_value::<RuleAction<'arn, 'grm>>()),
                ),
            ),
            "InputLiteral" => RuleAction::InputLiteral(parse_string(args[0], src)),
            "Name" => RuleAction::Name(parse_identifier(args[0], src)),
            "Value" => RuleAction::Value(args[0]),
            _ => unreachable!(),
        }
    }
}
