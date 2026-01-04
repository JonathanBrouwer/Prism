use std::sync::Arc;

use crate::core::allocs::alloc_extend;
use crate::parsable::Parsable;
use crate::parsable::parsed::Parsed;
use crate::parser::parsed_list::ParsedList;
use prism_input::input::Input;
use prism_input::input_table::InputTable;
use prism_input::span::Span;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
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

    fn from_construct(
        _span: Span,
        constructor: &str,
        args: &[Parsed],
        _env: &mut Db,
        input: &InputTable,
    ) -> Self {
        match constructor {
            "Construct" => {
                let parsed = &args[1];
                RuleAction::Construct {
                    ns: args[0].value_ref::<Input>().clone(),
                    name: parsed.value_ref::<Input>().clone(),
                    args: alloc_extend(
                        args[2]
                            .value_ref::<ParsedList>()
                            .iter()
                            .map(|((), v)| v)
                            .map(|sub| sub.value_cloned::<RuleAction>()),
                    ),
                }
            }
            "InputLiteral" => {
                RuleAction::InputLiteral(args[0].value_ref::<Input>().parse_escaped_string(input))
            }
            "Name" => {
                let parsed = &args[0];
                RuleAction::Name(parsed.value_ref::<Input>().clone())
            }
            "Value" => {
                let parsed = &args[0];
                RuleAction::Value {
                    ns: parsed.value_ref::<Input>().clone(),
                    value: args[1].clone(),
                }
            }
            _ => unreachable!(),
        }
    }
}
