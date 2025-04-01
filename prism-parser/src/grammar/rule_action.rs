use std::sync::Arc;

use crate::core::allocs::alloc_extend;
use crate::core::input::Input;
use crate::core::span::Span;
use crate::parsable::Parsable;
use crate::parsable::parsed::Parsed;
use crate::parser::parsed_list::ParsedList;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum RuleAction {
    Name(Input),
    InputLiteral(Input),
    Construct {
        ns: Input,
        name: Input,
        args: Arc<[Arc<Self>]>,
    },
    #[serde(skip)]
    Value {
        ns: Input,
        value: Parsed,
    },
}

impl<Db> Parsable<Db> for RuleAction {
    type EvalCtx = ();

    fn from_construct(_span: Span, constructor: &Input, args: &[Parsed], _env: &mut Db) -> Self {
        match constructor.as_str() {
            "Construct" => RuleAction::Construct {
                ns: Input::from_parsed(&args[0]),
                name: Input::from_parsed(&args[1]),
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
            "Name" => RuleAction::Name(Input::from_parsed(&args[0])),
            "Value" => RuleAction::Value {
                ns: Input::from_parsed(&args[0]),
                value: args[1].clone(),
            },
            _ => unreachable!(),
        }
    }
}
