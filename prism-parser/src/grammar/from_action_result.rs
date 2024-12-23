use crate::core::cache::Allocs;
use crate::core::input::Input;
use crate::grammar::escaped_string::EscapedString;
use crate::grammar::rule_action::RuleAction;
use crate::grammar::rule_annotation::RuleAnnotation;
use crate::grammar::rule_expr::RuleExpr;
use crate::grammar::{Block, GrammarFile, Rule};
use crate::grammar::annotated_rule_expr::AnnotatedRuleExpr;
use crate::parsable::action_result::ActionResult;
use crate::parsable::action_result::ActionResult::*;
use crate::parsable::parsed::Parsed;
use crate::parser::parsed_list::ParsedList;

#[macro_export]
macro_rules! result_match {
    {match $e1:expr => $p1:pat_param, $(match $es:expr => $ps:pat_param,)* create $body:expr} => {
        match $e1 {
            $p1 => {
                result_match! { $(match $es => $ps,)* create $body }
            },
            _ => None,
        }
    };
    {create $body:expr} => {
        Some($body)
    };
}

pub fn parse_grammarfile<'arn, 'grm: 'arn>(
    r: Parsed<'arn, 'grm>,
    src: &'grm str,
    allocs: Allocs<'arn>,
    parse_a: impl Fn(Parsed<'arn, 'grm>) -> RuleAction<'arn, 'grm>,
) -> Option<GrammarFile<'arn, 'grm>> {
    result_match! {
        match r.into_value::<ActionResult<'arn, 'grm>>() => Construct(_, "GrammarFile", rules),
        match &rules[..] => [rules],
        create GrammarFile {
            rules: allocs.try_alloc_extend(rules.into_value::<ParsedList>().into_iter().map(|rule| parse_rule(rule.into_value::<ActionResult>(), src, allocs, &parse_a)))?,
        }
    }
}

fn parse_rule<'arn, 'grm: 'arn>(
    r: &'arn ActionResult<'arn, 'grm>,
    src: &'grm str,
    allocs: Allocs<'arn>,
    parse_a: &impl Fn(Parsed<'arn, 'grm>) -> RuleAction<'arn, 'grm>,
) -> Option<Rule<'arn, 'grm>> {
    result_match! {
        match r => Construct(_, "Rule", rule_body),
        match &rule_body[..] => [name, extend, args, blocks],
        create Rule {
            name: parse_identifier(*name, src),
            adapt: extend.into_value::<ParsedList>().into_iter().next().is_some(),
            args: allocs.alloc_extend(args.into_value::<ParsedList>().into_iter().map(|n| ("ActionResult", parse_identifier(n, src)))),
            blocks: allocs.try_alloc_extend(blocks.into_value::<ParsedList>().into_iter().map(|block| parse_block(block.into_value::<ActionResult>(), src, allocs, parse_a)))?,
        return_type: "ActionResult",}
    }
}

fn parse_block<'arn, 'grm: 'arn>(
    r: &'arn ActionResult<'arn, 'grm>,
    src: &'grm str,
    allocs: Allocs<'arn>,
    parse_a: &impl Fn(Parsed<'arn, 'grm>) -> RuleAction<'arn, 'grm>,
) -> Option<Block<'arn, 'grm>> {
    result_match! {
        match r => Construct(_, "Block", b),
        match &b[..] => [name, extend, cs],
        create Block { name: parse_identifier(*name, src),
            adapt: extend.into_value::<ParsedList>().into_iter().next().is_some(),
            constructors: parse_constructors(*cs, src, allocs, parse_a)? }
    }
}

fn parse_constructors<'arn, 'grm: 'arn>(
    r: Parsed<'arn, 'grm>,
    src: &'grm str,
    allocs: Allocs<'arn>,
    parse_a: &impl Fn(Parsed<'arn, 'grm>) -> RuleAction<'arn, 'grm>,
) -> Option<&'arn [AnnotatedRuleExpr<'arn, 'grm>]> {
    result_match! {
        create allocs.alloc_extend(r.into_value::<ParsedList>().into_iter().map(|c| *c.into_value::<AnnotatedRuleExpr>()))
    }
}

pub(crate) fn parse_identifier<'grm>(r: Parsed<'_, 'grm>, src: &'grm str) -> &'grm str {
    r.into_value::<Input<'grm>>().as_str(src)
}

pub(crate) fn parse_string<'arn, 'grm>(
    r: Parsed<'arn, 'grm>,
    src: &'grm str,
) -> EscapedString<'grm> {
    let Input::Value(span) = r.into_value::<Input<'grm>>() else {
        panic!()
    };
    EscapedString::from_escaped(&src[*span])
}

pub fn parse_option<'arn, 'grm, T>(
    r: &ActionResult<'arn, 'grm>,
    src: &str,
    sub: impl Fn(Parsed<'arn, 'grm>, &str) -> T,
) -> Option<T> {
    match r {
        Construct(_, "None", []) => None,
        Construct(_, "Some", b) => Some(sub(b[0], src)),
        _ => unreachable!(),
    }
}

pub fn parse_u64(r: Parsed, src: &str) -> u64 {
    r.into_value::<Input>().as_cow(src).parse().unwrap()
}
