use crate::core::allocs::Allocs;
use crate::core::input::Input;
use crate::core::input_table::InputTable;
use crate::core::span::Span;
use crate::grammar::from_action_result::parse_identifier;
use crate::grammar::serde_leak::*;
use crate::parsable::parsed::Parsed;
use crate::parsable::{Parsable, ParseResult};
use crate::parser::parsed_list::ParsedList;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum RuleAction<'arn> {
    Name(&'arn str),
    InputLiteral(Input),
    Construct {
        ns: &'arn str,
        name: &'arn str,
        #[serde(with = "leak_slice")]
        args: &'arn [Self],
    },
    #[serde(skip)]
    Value {
        ns: &'arn str,
        value: Parsed<'arn>,
    },
}

impl ParseResult for RuleAction<'_> {}
impl<'arn, Env> Parsable<'arn, Env> for RuleAction<'arn> {
    type EvalCtx = ();

    fn from_construct(
        _span: Span,
        constructor: &'arn str,
        args: &[Parsed<'arn>],
        allocs: Allocs<'arn>,
        src: &InputTable<'arn>,
        _env: &mut Env,
    ) -> Self {
        match constructor {
            "Construct" => RuleAction::Construct {
                ns: parse_identifier(args[0], src),
                name: parse_identifier(args[1], src),
                args: allocs.alloc_extend(
                    args[2]
                        .into_value::<ParsedList>()
                        .into_iter()
                        .map(|((), v)| v)
                        .map(|sub| *sub.into_value::<RuleAction<'arn>>()),
                ),
            },
            "InputLiteral" => {
                RuleAction::InputLiteral(args[0].into_value::<Input>().parse_escaped_string())
            }
            "Name" => RuleAction::Name(parse_identifier(args[0], src)),
            "Value" => RuleAction::Value {
                ns: parse_identifier(args[0], src),
                value: args[1],
            },
            _ => unreachable!(),
        }
    }
}
