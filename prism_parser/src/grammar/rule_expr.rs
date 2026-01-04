use crate::core::allocs::alloc_extend;
use crate::grammar::charclass::CharClass;
use crate::grammar::rule_action::RuleAction;
use crate::parsable::Parsable;
use crate::parsable::parsed::Parsed;
use crate::parser::parsed_list::ParsedList;
use prism_input::input::Input;
use prism_input::input_table::InputTable;
use prism_input::span::Span;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug)]
pub enum RuleExpr {
    RunVar {
        rule: Input,

        args: Arc<[Arc<Self>]>,
    },
    CharClass(Arc<CharClass>),
    Literal(Input),
    Repeat {
        expr: Arc<Self>,
        min: u64,
        max: Option<u64>,

        delim: Arc<Self>,
    },
    Sequence(Arc<[Arc<Self>]>),
    Choice(Arc<[Arc<Self>]>),
    NameBind(Input, Arc<Self>),
    Action(Arc<Self>, Arc<RuleAction>),
    SliceInput(Arc<Self>),
    PosLookahead(Arc<Self>),
    NegLookahead(Arc<Self>),
    AtAdapt {
        ns: Input,
        name: Input,

        expr: Arc<Self>,
    },
}

impl<Db> Parsable<Db> for RuleExpr {
    type EvalCtx = ();

    fn from_construct(
        _span: Span,
        constructor: &str,
        args: &[Parsed],
        _env: &mut Db,
        input: &InputTable,
    ) -> Self {
        match constructor {
            "Action" => RuleExpr::Action(args[0].value_cloned(), args[1].value_cloned()),
            "Choice" => RuleExpr::Choice(alloc_extend(
                args[0]
                    .value_ref::<ParsedList>()
                    .iter()
                    .map(|((), v)| v)
                    .map(|sub| sub.value_cloned::<RuleExpr>()),
            )),
            "Sequence" => RuleExpr::Sequence(alloc_extend(
                args[0]
                    .value_ref::<ParsedList>()
                    .iter()
                    .map(|((), v)| v)
                    .map(|sub| sub.value_cloned::<RuleExpr>()),
            )),
            "NameBind" => {
                let parsed = &args[0];
                RuleExpr::NameBind(
                    parsed.value_ref::<Input>().clone(),
                    args[1].value_cloned::<RuleExpr>(),
                )
            }
            "Repeat" => RuleExpr::Repeat {
                expr: args[0].value_cloned::<RuleExpr>(),
                min: args[1].value_ref::<Input>().as_str(input).parse().unwrap(),
                max: *args[2].value_ref::<Option<u64>>(),
                delim: args[3].value_cloned::<RuleExpr>(),
            },
            "Literal" => {
                RuleExpr::Literal(args[0].value_ref::<Input>().parse_escaped_string(input))
            }
            "CharClass" => RuleExpr::CharClass(args[0].value_cloned::<CharClass>()),
            "SliceInput" => RuleExpr::SliceInput(args[0].value_cloned::<RuleExpr>()),
            "PosLookahead" => RuleExpr::PosLookahead(args[0].value_cloned::<RuleExpr>()),
            "NegLookahead" => RuleExpr::NegLookahead(args[0].value_cloned::<RuleExpr>()),
            "RunVar" => {
                let parsed = &args[0];
                RuleExpr::RunVar {
                    rule: parsed.value_ref::<Input>().clone(),
                    args: alloc_extend(
                        args[1]
                            .value_ref::<ParsedList>()
                            .iter()
                            .map(|((), v)| v)
                            .map(|sub| sub.value_cloned::<RuleExpr>()),
                    ),
                }
            }
            "AtAdapt" => {
                let parsed = &args[1];
                RuleExpr::AtAdapt {
                    ns: args[0].value_ref::<Input>().clone(),
                    name: parsed.value_ref::<Input>().clone(),
                    expr: args[2].value_cloned::<RuleExpr>(),
                }
            }
            _ => unreachable!(),
        }
    }
}
