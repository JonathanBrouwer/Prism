use std::sync::Arc;

use crate::core::allocs::alloc_extend;
use crate::core::input::Input;
use crate::core::input_table::InputTable;
use crate::core::span::Span;
use crate::grammar::identifier::{Identifier, parse_identifier};
use crate::parsable::Parsable;
use crate::parsable::parsed::Parsed;
use crate::parser::parsed_list::ParsedList;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum RuleAction {
    Name(Identifier),
    InputLiteral(Input),
    Construct {
        ns: Identifier,
        name: Identifier,
        args: Arc<[Arc<Self>]>,
    },
    #[serde(skip)]
    Value {
        ns: Identifier,
        value: Parsed,
    },
}

impl<Env> Parsable<Env> for RuleAction {
    type EvalCtx = ();

    fn from_construct(
        _span: Span,
        constructor: Identifier,
        args: &[Parsed],
        src: &InputTable,
        _env: &mut Env,
    ) -> Self {
        match constructor.as_str(src) {
            "Construct" => RuleAction::Construct {
                ns: parse_identifier(&args[0]),
                name: parse_identifier(&args[1]),
                args: alloc_extend(
                    args[2]
                        .value_ref::<ParsedList>()
                        .iter()
                        .map(|((), v)| v)
                        .map(|sub| sub.value_cloned::<RuleAction>()),
                ),
            },
            "InputLiteral" => {
                RuleAction::InputLiteral(args[0].value_ref::<Input>().parse_escaped_string())
            }
            "Name" => RuleAction::Name(parse_identifier(&args[0])),
            "Value" => RuleAction::Value {
                ns: parse_identifier(&args[0]),
                value: args[1].clone(),
            },
            _ => unreachable!(),
        }
    }
}
