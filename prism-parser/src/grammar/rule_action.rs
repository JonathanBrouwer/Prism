use crate::core::allocs::Allocs;
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
    Value(Parsed<'arn, 'grm>),
}

impl<'arn, 'grm: 'arn> ParseResult<'arn, 'grm> for RuleAction<'arn, 'grm> {}
impl<'arn, 'grm: 'arn, Env> Parsable<'arn, 'grm, Env> for RuleAction<'arn, 'grm> {
    type EvalCtx = ();

    fn from_construct(
        _span: Span,
        constructor: &'grm str,
        _args: &[Parsed<'arn, 'grm>],
        _allocs: Allocs<'arn>,
        _src: &'grm str,
        _env: &mut Env,
    ) -> Self {
        match constructor {
            "Construct" => RuleAction::Construct {
                ns: parse_identifier(_args[0], _src),
                name: parse_identifier(_args[1], _src),
                args: _allocs.alloc_extend(
                    _args[2]
                        .into_value::<ParsedList>()
                        .into_iter()
                        .map(|((), v)| v)
                        .map(|sub| *sub.into_value::<RuleAction<'arn, 'grm>>()),
                ),
            },
            "InputLiteral" => RuleAction::InputLiteral(parse_string(_args[0], _src)),
            "Name" => RuleAction::Name(parse_identifier(_args[0], _src)),
            "Value" => RuleAction::Value(_args[0]),
            _ => unreachable!(),
        }
    }
}
