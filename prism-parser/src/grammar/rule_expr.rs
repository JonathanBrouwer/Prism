use crate::core::cache::Allocs;
use crate::core::input::Input;
use crate::core::span::Span;
use crate::grammar::charclass::CharClass;
use crate::grammar::escaped_string::EscapedString;
use crate::grammar::from_action_result::{parse_identifier, parse_string};
use crate::grammar::rule_action::RuleAction;
use crate::grammar::serde_leak::*;
use crate::parsable::parsed::Parsed;
use crate::parsable::{Parsable2, ParseResult};
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
    AtAdapt(&'grm str, #[serde(with = "leak")] &'arn Self),
    Guid,
}

impl<'arn, 'grm: 'arn> ParseResult<'arn, 'grm> for RuleExpr<'arn, 'grm> {}
impl<'arn, 'grm: 'arn, Env> Parsable2<'arn, 'grm, Env> for RuleExpr<'arn, 'grm> {
    fn from_construct(
        _span: Span,
        constructor: &'grm str,
        _args: &[Parsed<'arn, 'grm>],
        _allocs: Allocs<'arn>,
        _src: &'grm str,
        _env: &mut Env,
    ) -> Result<Self, String> {
        Ok(match constructor {
            "Action" => RuleExpr::Action(
                _allocs.alloc(_args[0].into_value()),
                _allocs.alloc(_args[1].into_value()),
            ),
            "Choice" => RuleExpr::Choice(
                _allocs.alloc_extend(
                    _args[0]
                        .into_value::<ParsedList>()
                        .into_iter()
                        .map(|sub| *sub.into_value::<RuleExpr>()),
                ),
            ),
            "Sequence" => RuleExpr::Sequence(
                _allocs.alloc_extend(
                    _args[0]
                        .into_value::<ParsedList>()
                        .into_iter()
                        .map(|sub| *sub.into_value::<RuleExpr>()),
                ),
            ),
            "NameBind" => RuleExpr::NameBind(
                parse_identifier(_args[0], _src),
                _args[1].into_value::<RuleExpr>(),
            ),
            "Repeat" => RuleExpr::Repeat {
                expr: _args[0].into_value::<RuleExpr>(),
                min: _args[1].into_value::<Input>().as_str(_src).parse().unwrap(),
                max: *_args[2].into_value::<Option<u64>>(),
                delim: _args[3].into_value::<RuleExpr>(),
            },
            "Literal" => RuleExpr::Literal(parse_string(_args[0], _src)),
            "CharClass" => RuleExpr::CharClass(_args[0].into_value::<CharClass>()),
            "SliceInput" => RuleExpr::SliceInput(_args[0].into_value::<RuleExpr>()),
            "PosLookahead" => RuleExpr::PosLookahead(_args[0].into_value::<RuleExpr>()),
            "NegLookahead" => RuleExpr::NegLookahead(_args[0].into_value::<RuleExpr>()),
            "Guid" => RuleExpr::Guid,
            "RunVar" => RuleExpr::RunVar {
                rule: parse_identifier(_args[0], _src),
                args: _allocs.alloc_extend(
                    _args[1]
                        .into_value::<ParsedList>()
                        .into_iter()
                        .map(|sub| *sub.into_value::<RuleExpr>()),
                ),
            },
            "AtAdapt" => RuleExpr::AtAdapt(
                parse_identifier(_args[0], _src),
                _args[1].into_value::<RuleExpr>(),
            ),
            _ => unreachable!(),
        })
    }
}
