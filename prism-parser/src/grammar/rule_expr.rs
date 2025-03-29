use crate::core::allocs::Allocs;
use crate::core::input::Input;
use crate::core::input_table::InputTable;
use crate::core::span::Span;
use crate::grammar::charclass::CharClass;
use crate::grammar::identifier::parse_identifier_old;
use crate::grammar::rule_action::RuleAction;
use crate::grammar::serde_leak::*;
use crate::parsable::parsed::Parsed;
use crate::parsable::{Parsable, ParseResult};
use crate::parser::parsed_list::ParsedList;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize, Debug)]
pub enum RuleExpr<'arn> {
    RunVar {
        rule: &'arn str,
        #[serde(with = "leak_slice")]
        args: &'arn [RuleExpr<'arn>],
    },
    CharClass(#[serde(with = "leak")] &'arn CharClass<'arn>),
    Literal(Input),
    Repeat {
        #[serde(with = "leak")]
        expr: &'arn Self,
        min: u64,
        max: Option<u64>,
        #[serde(with = "leak")]
        delim: &'arn Self,
    },
    Sequence(#[serde(with = "leak_slice")] &'arn [RuleExpr<'arn>]),
    Choice(#[serde(with = "leak_slice")] &'arn [RuleExpr<'arn>]),
    NameBind(&'arn str, #[serde(with = "leak")] &'arn Self),
    Action(
        #[serde(with = "leak")] &'arn Self,
        #[serde(with = "leak")] &'arn RuleAction<'arn>,
    ),
    SliceInput(#[serde(with = "leak")] &'arn Self),
    PosLookahead(#[serde(with = "leak")] &'arn Self),
    NegLookahead(#[serde(with = "leak")] &'arn Self),
    AtAdapt {
        ns: &'arn str,
        name: &'arn str,
        #[serde(with = "leak")]
        expr: &'arn Self,
    },
    Guid,
}

impl ParseResult for RuleExpr<'_> {}
impl<'arn, Env> Parsable<'arn, Env> for RuleExpr<'arn> {
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
            "Action" => RuleExpr::Action(args[0].into_value(), args[1].into_value()),
            "Choice" => RuleExpr::Choice(
                allocs.alloc_extend(
                    args[0]
                        .into_value::<ParsedList>()
                        .into_iter()
                        .map(|((), v)| v)
                        .map(|sub| *sub.into_value::<RuleExpr>()),
                ),
            ),
            "Sequence" => RuleExpr::Sequence(
                allocs.alloc_extend(
                    args[0]
                        .into_value::<ParsedList>()
                        .into_iter()
                        .map(|((), v)| v)
                        .map(|sub| *sub.into_value::<RuleExpr>()),
                ),
            ),
            "NameBind" => RuleExpr::NameBind(
                parse_identifier_old(args[0], src),
                args[1].into_value::<RuleExpr>(),
            ),
            "Repeat" => RuleExpr::Repeat {
                expr: args[0].into_value::<RuleExpr>(),
                min: args[1].into_value::<Input>().as_str(src).parse().unwrap(),
                max: *args[2].into_value::<Option<u64>>(),
                delim: args[3].into_value::<RuleExpr>(),
            },
            "Literal" => RuleExpr::Literal(args[0].into_value::<Input>().parse_escaped_string()),
            "CharClass" => RuleExpr::CharClass(args[0].into_value::<CharClass>()),
            "SliceInput" => RuleExpr::SliceInput(args[0].into_value::<RuleExpr>()),
            "PosLookahead" => RuleExpr::PosLookahead(args[0].into_value::<RuleExpr>()),
            "NegLookahead" => RuleExpr::NegLookahead(args[0].into_value::<RuleExpr>()),
            "Guid" => RuleExpr::Guid,
            "RunVar" => RuleExpr::RunVar {
                rule: parse_identifier_old(args[0], src),
                args: allocs.alloc_extend(
                    args[1]
                        .into_value::<ParsedList>()
                        .into_iter()
                        .map(|((), v)| v)
                        .map(|sub| *sub.into_value::<RuleExpr>()),
                ),
            },
            "AtAdapt" => RuleExpr::AtAdapt {
                ns: parse_identifier_old(args[0], src),
                name: parse_identifier_old(args[1], src),
                expr: args[2].into_value::<RuleExpr>(),
            },
            _ => unreachable!(),
        }
    }
}
