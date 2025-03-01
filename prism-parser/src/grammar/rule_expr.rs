use crate::core::cache::Allocs;
use crate::core::input::Input;
use crate::core::span::Span;
use crate::grammar::charclass::CharClass;
use crate::grammar::escaped_string::EscapedString;
use crate::grammar::from_action_result::{parse_identifier, parse_string};
use crate::grammar::rule_action::RuleAction;
use crate::grammar::serde_leak::*;
use crate::parsable::parsed::Parsed;
use crate::parsable::{Parsable, ParseResult};
use crate::parser::parsed_list::ParsedList;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum RuleExpr<'arn, 'grm> {
    RunVar {
        rule: &'grm str,
        #[serde(with = "leak_slice")]
        args: &'arn [RuleExpr<'arn, 'grm>],
    },
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
    AtAdapt(&'grm str, &'grm str, #[serde(with = "leak")] &'arn Self),
    Guid,
}

impl<'arn, 'grm: 'arn> ParseResult<'arn, 'grm> for RuleExpr<'arn, 'grm> {}
impl<'arn, 'grm: 'arn, Env> Parsable<'arn, 'grm, Env> for RuleExpr<'arn, 'grm> {
    type EvalCtx = ();

    fn from_construct(
        _span: Span,
        constructor: &'grm str,
        args: &[Parsed<'arn, 'grm>],
        allocs: Allocs<'arn>,
        src: &'grm str,
        _env: &mut Env,
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
                min: args[1].into_value::<Input>().as_str(src).parse().unwrap(),
                max: *args[2].into_value::<Option<u64>>(),
                delim: args[3].into_value::<RuleExpr>(),
            },
            "Literal" => RuleExpr::Literal(parse_string(args[0], src)),
            "CharClass" => RuleExpr::CharClass(args[0].into_value::<CharClass>()),
            "SliceInput" => RuleExpr::SliceInput(args[0].into_value::<RuleExpr>()),
            "PosLookahead" => RuleExpr::PosLookahead(args[0].into_value::<RuleExpr>()),
            "NegLookahead" => RuleExpr::NegLookahead(args[0].into_value::<RuleExpr>()),
            "Guid" => RuleExpr::Guid,
            "RunVar" => RuleExpr::RunVar {
                rule: parse_identifier(args[0], src),
                args: allocs.alloc_extend(
                    args[1]
                        .into_value::<ParsedList>()
                        .into_iter()
                        .map(|sub| *sub.into_value::<RuleExpr>()),
                ),
            },
            "AtAdapt" => RuleExpr::AtAdapt(
                parse_identifier(args[0], src),
                parse_identifier(args[1], src),
                args[2].into_value::<RuleExpr>(),
            ),
            _ => unreachable!(),
        }
    }
}
