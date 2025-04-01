use crate::core::allocs::alloc_extend;
use crate::core::input::Input;
use crate::core::span::Span;
use crate::grammar::charclass::CharClass;
use crate::grammar::rule_action::RuleAction;
use crate::parsable::Parsable;
use crate::parsable::parsed::Parsed;
use crate::parser::parsed_list::ParsedList;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
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
    Guid,
}

impl<Db> Parsable<Db> for RuleExpr {
    type EvalCtx = ();

    fn from_construct(_span: Span, constructor: &Input, args: &[Parsed], _env: &mut Db) -> Self {
        match constructor.as_str() {
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
            "NameBind" => RuleExpr::NameBind(
                Input::from_parsed(&args[0]),
                args[1].value_cloned::<RuleExpr>(),
            ),
            "Repeat" => RuleExpr::Repeat {
                expr: args[0].value_cloned::<RuleExpr>(),
                min: args[1].value_ref::<Input>().as_str().parse().unwrap(),
                max: *args[2].value_ref::<Option<u64>>(),
                delim: args[3].value_cloned::<RuleExpr>(),
            },
            "Literal" => RuleExpr::Literal(args[0].value_ref::<Input>().parse_escaped_string()),
            "CharClass" => RuleExpr::CharClass(args[0].value_cloned::<CharClass>()),
            "SliceInput" => RuleExpr::SliceInput(args[0].value_cloned::<RuleExpr>()),
            "PosLookahead" => RuleExpr::PosLookahead(args[0].value_cloned::<RuleExpr>()),
            "NegLookahead" => RuleExpr::NegLookahead(args[0].value_cloned::<RuleExpr>()),
            "Guid" => RuleExpr::Guid,
            "RunVar" => RuleExpr::RunVar {
                rule: Input::from_parsed(&args[0]),
                args: alloc_extend(
                    args[1]
                        .value_ref::<ParsedList>()
                        .iter()
                        .map(|((), v)| v)
                        .map(|sub| sub.value_cloned::<RuleExpr>()),
                ),
            },
            "AtAdapt" => RuleExpr::AtAdapt {
                ns: Input::from_parsed(&args[0]),
                name: Input::from_parsed(&args[1]),
                expr: args[2].value_cloned::<RuleExpr>(),
            },
            _ => unreachable!(),
        }
    }
}
