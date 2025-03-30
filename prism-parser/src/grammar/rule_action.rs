use crate::core::allocs::Allocs;
use crate::core::input::Input;
use crate::core::input_table::InputTable;
use crate::core::span::Span;
use crate::grammar::identifier::{Identifier, parse_identifier};
use crate::grammar::serde_leak::*;
use crate::parsable::parsed::Parsed;
use crate::parsable::{Parsable, ParseResult};
use crate::parser::parsed_list::ParsedList;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum RuleAction<'arn> {
    Name(Identifier),
    InputLiteral(Input),
    Construct {
        ns: Identifier,
        name: Identifier,
        #[serde(with = "leak_slice", borrow)]
        args: &'arn [Self],
    },
    #[serde(skip)]
    Value {
        ns: Identifier,
        value: Parsed<'arn>,
    },
}

impl ParseResult for RuleAction<'_> {}
impl<'arn, Env> Parsable<'arn, Env> for RuleAction<'arn> {
    type EvalCtx = ();

    fn from_construct(
        _span: Span,
        constructor: Identifier,
        args: &[Parsed<'arn>],
        allocs: Allocs<'arn>,
        src: &InputTable<'arn>,
        _env: &mut Env,
    ) -> Self {
        match constructor.as_str(src) {
            "Construct" => RuleAction::Construct {
                ns: parse_identifier(args[0]),
                name: parse_identifier(args[1]),
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
            "Name" => RuleAction::Name(parse_identifier(args[0])),
            "Value" => RuleAction::Value {
                ns: parse_identifier(args[0]),
                value: args[1],
            },
            _ => unreachable!(),
        }
    }
}
