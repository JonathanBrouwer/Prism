use crate::core::allocs::Allocs;
use crate::core::input_table::InputTable;
use crate::core::span::Span;
use crate::grammar::escaped_string::EscapedString;
use crate::grammar::from_action_result::{parse_identifier, parse_string};
use crate::grammar::serde_leak::*;
use crate::parsable::parsed::Parsed;
use crate::parsable::{Parsable, ParseResult};
use crate::parser::parsed_list::ParsedList;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum RuleAction<'arn, 'grm> {
    Name(&'grm str),
    InputLiteral(EscapedString<'grm>),
    Construct {
        ns: &'grm str,
        name: &'grm str,
        #[serde(with = "leak_slice")]
        args: &'arn [Self],
    },
    #[serde(skip)]
    Value {
        ns: &'grm str,
        value: Parsed<'arn, 'grm>,
    },
}

impl<'arn, 'grm: 'arn> ParseResult<'arn, 'grm> for RuleAction<'arn, 'grm> {}
impl<'arn, 'grm: 'arn, Env> Parsable<'arn, 'grm, Env> for RuleAction<'arn, 'grm> {
    type EvalCtx = ();

    fn from_construct(
        _span: Span,
        constructor: &'grm str,
        args: &[Parsed<'arn, 'grm>],
        _allocs: Allocs<'arn>,
        src: &InputTable<'grm>,
        _env: &mut Env,
    ) -> Self {
        match constructor {
            "Construct" => RuleAction::Construct {
                ns: parse_identifier(args[0], src),
                name: parse_identifier(args[1], src),
                args: _allocs.alloc_extend(
                    args[2]
                        .into_value::<ParsedList>()
                        .into_iter()
                        .map(|((), v)| v)
                        .map(|sub| *sub.into_value::<RuleAction<'arn, 'grm>>()),
                ),
            },
            "InputLiteral" => RuleAction::InputLiteral(parse_string(args[0], src)),
            "Name" => RuleAction::Name(parse_identifier(args[0], src)),
            "Value" => RuleAction::Value {
                ns: parse_identifier(args[0], src),
                value: args[1],
            },
            _ => unreachable!(),
        }
    }
}
