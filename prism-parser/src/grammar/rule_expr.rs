use crate::core::cache::Allocs;
use crate::core::span::Span;
use crate::grammar::charclass::CharClass;
use crate::grammar::escaped_string::EscapedString;
use crate::grammar::from_action_result::{parse_identifier, parse_option, parse_string, parse_u64};
use crate::grammar::rule_action::RuleAction;
use crate::grammar::serde_leak::*;
use crate::parsable::action_result::ActionResult;
use crate::parsable::parsed::Parsed;
use crate::parsable::Parsable;
use crate::parser::parsed_list::ParsedList;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum RuleExpr<'arn, 'grm> {
    RunVar(
        &'grm str,
        #[serde(with = "leak_slice")] &'arn [RuleExpr<'arn, 'grm>],
    ),
    CharClass(#[serde(with = "leak")] &'arn CharClass<'arn>),
    Literal(EscapedString<'grm>),
    Repeat {
        #[serde(with = "leak")]
        expr: &'arn Self,
        min: u64,
        max: Option<u64>,
        #[serde(with = "leak")]
        delim: &'arn Self,
    },
    Sequence(#[serde(with = "leak_slice")] &'arn [RuleExpr<'arn, 'grm>]),
    Choice(#[serde(with = "leak_slice")] &'arn [RuleExpr<'arn, 'grm>]),
    NameBind(&'grm str, #[serde(with = "leak")] &'arn Self),
    Action(
        #[serde(with = "leak")] &'arn Self,
        #[serde(with = "leak")] &'arn RuleAction<'arn, 'grm>,
    ),
    SliceInput(#[serde(with = "leak")] &'arn Self),
    PosLookahead(#[serde(with = "leak")] &'arn Self),
    NegLookahead(#[serde(with = "leak")] &'arn Self),
    This,
    Next,
    AtAdapt(&'grm str, &'grm str),
    Guid,
}

impl<'arn, 'grm: 'arn> Parsable<'arn, 'grm> for RuleExpr<'arn, 'grm> {
    fn from_construct(
        _span: Span,
        constructor: &'grm str,
        args: &[Parsed<'arn, 'grm>],
        allocs: Allocs<'arn>,
        src: &'grm str,
    ) -> Self {
        match constructor {
            "Action" => RuleExpr::Action(
                allocs.alloc(args[0].into_value()),
                allocs.alloc(args[1].into_value()),
            ),
            "Choice" => RuleExpr::Choice(
                allocs.alloc_extend(
                    args[0]
                        .into_value::<ParsedList>()
                        .into_iter()
                        .map(|sub| *sub.into_value::<RuleExpr>()),
                ),
            ),
            "Sequence" => RuleExpr::Sequence(
                allocs.alloc_extend(
                    args[0]
                        .into_value::<ParsedList>()
                        .into_iter()
                        .map(|sub| *sub.into_value::<RuleExpr>()),
                ),
            ),
            "NameBind" => RuleExpr::NameBind(
                parse_identifier(args[0], src),
                args[1].into_value::<RuleExpr>(),
            ),
            "Repeat" => RuleExpr::Repeat {
                expr: args[0].into_value::<RuleExpr>(),
                min: parse_u64(args[1], src),
                max: parse_option(args[2].into_value::<ActionResult>(), src, parse_u64),
                delim: args[3].into_value::<RuleExpr>(),
            },
            "Literal" => RuleExpr::Literal(parse_string(args[0], src)),
            "CharClass" => RuleExpr::CharClass(args[0].into_value::<CharClass>()),
            "SliceInput" => RuleExpr::SliceInput(args[0].into_value::<RuleExpr>()),
            "PosLookahead" => RuleExpr::PosLookahead(args[0].into_value::<RuleExpr>()),
            "NegLookahead" => RuleExpr::NegLookahead(args[0].into_value::<RuleExpr>()),
            "This" => RuleExpr::This,
            "Next" => RuleExpr::Next,
            "Guid" => RuleExpr::Guid,
            "RunVar" => RuleExpr::RunVar(
                parse_identifier(args[0], src),
                allocs.alloc_extend(
                    args[1]
                        .into_value::<ParsedList>()
                        .into_iter()
                        .map(|sub| *sub.into_value::<RuleExpr>()),
                ),
            ),
            "AtAdapt" => RuleExpr::AtAdapt(
                parse_identifier(args[0], src),
                parse_identifier(args[1], src),
            ),
            _ => unreachable!(),
        }
    }
}
