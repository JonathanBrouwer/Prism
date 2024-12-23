use crate::core::cache::Allocs;
use crate::core::input::Input;
use crate::grammar::charclass::{CharClass, CharClassRange};
use crate::grammar::escaped_string::EscapedString;
use crate::grammar::rule_action::RuleAction;
use crate::grammar::rule_annotation::RuleAnnotation;
use crate::grammar::{AnnotatedRuleExpr, Block, GrammarFile, Rule, RuleExpr};
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
        create allocs.try_alloc_extend(r.into_value::<ParsedList>().into_iter().map(|c| parse_annotated_rule_expr(c.into_value::<ActionResult<'arn, 'grm>>(), src, allocs, parse_a)))?
    }
}

fn parse_annotated_rule_expr<'arn, 'grm: 'arn>(
    r: &'arn ActionResult<'arn, 'grm>,
    src: &'grm str,
    allocs: Allocs<'arn>,
    parse_a: &impl Fn(Parsed<'arn, 'grm>) -> RuleAction<'arn, 'grm>,
) -> Option<AnnotatedRuleExpr<'arn, 'grm>> {
    result_match! {
        match r => Construct(_, "AnnotatedExpr", body),
        match &body[..] => [annots, e],
        create AnnotatedRuleExpr(allocs.alloc_extend(annots.into_value::<ParsedList>().into_iter().map(|annot| *annot.into_value::<RuleAnnotation>())), parse_rule_expr(e.into_value::<ActionResult>(), src, allocs, parse_a)?)
    }
}

fn parse_rule_expr<'arn, 'grm: 'arn>(
    r: &'arn ActionResult<'arn, 'grm>,
    src: &'grm str,
    allocs: Allocs<'arn>,
    parse_a: &impl Fn(Parsed<'arn, 'grm>) -> RuleAction<'arn, 'grm>,
) -> Option<RuleExpr<'arn, 'grm>> {
    Some(match r {
        Construct(_, "Action", b) => RuleExpr::Action(
            allocs.alloc(parse_rule_expr(
                b[0].into_value::<ActionResult>(),
                src,
                allocs,
                parse_a,
            )?),
            parse_a(b[1]),
        ),
        Construct(_, "Choice", b) => RuleExpr::Choice(result_match! {
            create allocs.try_alloc_extend(b[0].into_value::<ParsedList>().into_iter().map(|sub| parse_rule_expr(sub.into_value::<ActionResult>(), src, allocs, parse_a)))?
        }?),
        Construct(_, "Sequence", b) => RuleExpr::Sequence(result_match! {
            create allocs.try_alloc_extend(b[0].into_value::<ParsedList>().into_iter().map(|sub| parse_rule_expr(sub.into_value::<ActionResult>(), src, allocs, parse_a)))?
        }?),
        Construct(_, "NameBind", b) => RuleExpr::NameBind(
            parse_identifier(b[0], src),
            allocs.alloc(parse_rule_expr(
                b[1].into_value::<ActionResult>(),
                src,
                allocs,
                parse_a,
            )?),
        ),
        Construct(_, "Repeat", b) => RuleExpr::Repeat {
            expr: allocs.alloc(parse_rule_expr(
                b[0].into_value::<ActionResult>(),
                src,
                allocs,
                parse_a,
            )?),
            min: parse_u64(b[1], src)?,
            max: parse_option(b[2].into_value::<ActionResult>(), src, parse_u64)?,
            delim: allocs.alloc(parse_rule_expr(
                b[3].into_value::<ActionResult>(),
                src,
                allocs,
                parse_a,
            )?),
        },
        Construct(_, "Literal", b) => RuleExpr::Literal(parse_string(b[0], src)),
        Construct(_, "CharClass", b) => RuleExpr::CharClass(*b[0].into_value::<CharClass>()),
        Construct(_, "SliceInput", b) => RuleExpr::SliceInput(allocs.alloc(parse_rule_expr(
            b[0].into_value::<ActionResult>(),
            src,
            allocs,
            parse_a,
        )?)),
        Construct(_, "PosLookahead", b) => RuleExpr::PosLookahead(allocs.alloc(parse_rule_expr(
            b[0].into_value::<ActionResult>(),
            src,
            allocs,
            parse_a,
        )?)),
        Construct(_, "NegLookahead", b) => RuleExpr::NegLookahead(allocs.alloc(parse_rule_expr(
            b[0].into_value::<ActionResult>(),
            src,
            allocs,
            parse_a,
        )?)),
        Construct(_, "This", _) => RuleExpr::This,
        Construct(_, "Next", _) => RuleExpr::Next,
        Construct(_, "Guid", _) => RuleExpr::Guid,
        Construct(_, "RunVar", b) => RuleExpr::RunVar(
            parse_identifier(b[0], src),
            result_match! {
                create allocs.try_alloc_extend(b[1].into_value::<ParsedList>().into_iter().map(|sub| {
                    parse_rule_expr(sub.into_value::<ActionResult>(), src, allocs, parse_a)
                }))?
            }?,
        ),
        Construct(_, "AtAdapt", b) => {
            RuleExpr::AtAdapt(parse_identifier(b[0], src), parse_identifier(b[1], src))
        }
        _ => return None,
    })
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

fn parse_option<'arn, 'grm, T>(
    r: &ActionResult<'arn, 'grm>,
    src: &str,
    sub: impl Fn(Parsed<'arn, 'grm>, &str) -> Option<T>,
) -> Option<Option<T>> {
    match r {
        Construct(_, "None", []) => Some(None),
        Construct(_, "Some", b) => Some(Some(sub(b[0], src)?)),
        _ => None,
    }
}

fn parse_u64(r: Parsed, src: &str) -> Option<u64> {
    r.try_into_value::<Input>()?.as_cow(src).parse().ok()
}
