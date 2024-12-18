use crate::core::cache::Allocs;
use crate::action::action_result::ActionResult;
use crate::action::action_result::ActionResult::*;
use crate::grammar::escaped_string::EscapedString;
use crate::grammar::rule_action::RuleAction;
use crate::grammar::{AnnotatedRuleExpr, Block, GrammarFile, Rule, RuleExpr};
use crate::grammar::{CharClass, RuleAnnotation};
use std::borrow::Cow;

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

pub fn parse_grammarfile<'arn_in, 'arn_out, 'grm>(
    r: &'arn_in ActionResult<'arn_in, 'grm>,
    src: &'grm str,
    allocs: Allocs<'arn_out>,
    parse_a: impl Fn(
        &'arn_in ActionResult<'arn_in, 'grm>,
        &'grm str,
    ) -> Option<RuleAction<'arn_out, 'grm>>,
) -> Option<GrammarFile<'arn_out, 'grm>> {
    result_match! {
        match r => Construct(_, "GrammarFile", rules),
        match &rules[..] => [rules],
        create GrammarFile {
            rules: allocs.try_alloc_extend(rules.iter_list().map(|rule| parse_rule(rule, src, allocs, &parse_a)))?,
        }
    }
}

fn parse_rule<'arn_in, 'arn_out, 'grm>(
    r: &'arn_in ActionResult<'arn_in, 'grm>,
    src: &'grm str,
    allocs: Allocs<'arn_out>,
    parse_a: &impl Fn(
        &'arn_in ActionResult<'arn_in, 'grm>,
        &'grm str,
    ) -> Option<RuleAction<'arn_out, 'grm>>,
) -> Option<Rule<'arn_out, 'grm>> {
    result_match! {
        match r => Construct(_, "Rule", rule_body),
        match &rule_body[..] => [name, extend, args, blocks],
        create Rule {
            name: parse_identifier(name, src)?,
            adapt: extend.iter_list().next().is_some(),
            args: allocs.try_alloc_extend(args.iter_list().map(|n| parse_identifier(n, src).map(|name| ("ActionResult", name))))?,
            blocks: allocs.try_alloc_extend(blocks.iter_list().map(|block| parse_block(block, src, allocs, parse_a)))?,
        return_type: "ActionResult",}
    }
}

fn parse_block<'arn_in, 'arn_out, 'grm>(
    r: &'arn_in ActionResult<'arn_in, 'grm>,
    src: &'grm str,
    allocs: Allocs<'arn_out>,
    parse_a: &impl Fn(
        &'arn_in ActionResult<'arn_in, 'grm>,
        &'grm str,
    ) -> Option<RuleAction<'arn_out, 'grm>>,
) -> Option<Block<'arn_out, 'grm>> {
    result_match! {
        match r => Construct(_, "Block", b),
        match &b[..] => [name, extend, cs],
        create Block { name: parse_identifier(name, src)?,
            adapt: extend.iter_list().next().is_some(),
            constructors: parse_constructors(cs, src, allocs, parse_a)? }
    }
}

fn parse_constructors<'arn_in, 'arn_out, 'grm>(
    r: &'arn_in ActionResult<'arn_in, 'grm>,
    src: &'grm str,
    allocs: Allocs<'arn_out>,
    parse_a: &impl Fn(
        &'arn_in ActionResult<'arn_in, 'grm>,
        &'grm str,
    ) -> Option<RuleAction<'arn_out, 'grm>>,
) -> Option<&'arn_out [AnnotatedRuleExpr<'arn_out, 'grm>]> {
    result_match! {
        create allocs.try_alloc_extend(r.iter_list().map(|c| parse_annotated_rule_expr(c, src, allocs, parse_a)))?
    }
}

fn parse_annotated_rule_expr<'arn_in, 'arn_out, 'grm>(
    r: &'arn_in ActionResult<'arn_in, 'grm>,
    src: &'grm str,
    allocs: Allocs<'arn_out>,
    parse_a: &impl Fn(
        &'arn_in ActionResult<'arn_in, 'grm>,
        &'grm str,
    ) -> Option<RuleAction<'arn_out, 'grm>>,
) -> Option<AnnotatedRuleExpr<'arn_out, 'grm>> {
    result_match! {
        match r => Construct(_, "AnnotatedExpr", body),
        match &body[..] => [annots, e],
        create AnnotatedRuleExpr(allocs.try_alloc_extend(annots.iter_list().map(|annot| parse_rule_annotation(annot, src)))?, parse_rule_expr(e, src, allocs, parse_a)?)
    }
}

fn parse_rule_annotation<'arn_in, 'grm>(
    r: &'arn_in ActionResult<'arn_in, 'grm>,
    src: &'grm str,
) -> Option<RuleAnnotation<'grm>> {
    Some(match r {
        Construct(_, "Error", b) => RuleAnnotation::Error(parse_string(&b[0], src)?),
        Construct(_, "DisableLayout", _) => RuleAnnotation::DisableLayout,
        Construct(_, "EnableLayout", _) => RuleAnnotation::EnableLayout,
        Construct(_, "DisableRecovery", _) => RuleAnnotation::DisableRecovery,
        Construct(_, "EnableRecovery", _) => RuleAnnotation::EnableRecovery,
        _ => return None,
    })
}

fn parse_rule_expr<'arn_in, 'arn_out, 'grm>(
    r: &'arn_in ActionResult<'arn_in, 'grm>,
    src: &'grm str,
    allocs: Allocs<'arn_out>,
    parse_a: &impl Fn(
        &'arn_in ActionResult<'arn_in, 'grm>,
        &'grm str,
    ) -> Option<RuleAction<'arn_out, 'grm>>,
) -> Option<RuleExpr<'arn_out, 'grm>> {
    Some(match r {
        Construct(_, "Action", b) => RuleExpr::Action(
            allocs.alloc(parse_rule_expr(&b[0], src, allocs, parse_a)?),
            parse_a(&b[1], src)?,
        ),
        Construct(_, "Choice", b) => RuleExpr::Choice(result_match! {
            create allocs.try_alloc_extend(b[0].iter_list().map(|sub| parse_rule_expr(sub, src, allocs, parse_a)))?
        }?),
        Construct(_, "Sequence", b) => RuleExpr::Sequence(result_match! {
            create allocs.try_alloc_extend(b[0].iter_list().map(|sub| parse_rule_expr(sub, src, allocs, parse_a)))?
        }?),
        Construct(_, "NameBind", b) => RuleExpr::NameBind(
            parse_identifier(&b[0], src)?,
            allocs.alloc(parse_rule_expr(&b[1], src, allocs, parse_a)?),
        ),
        Construct(_, "Repeat", b) => RuleExpr::Repeat {
            expr: allocs.alloc(parse_rule_expr(&b[0], src, allocs, parse_a)?),
            min: parse_u64(&b[1], src)?,
            max: parse_option(&b[2], src, parse_u64)?,
            delim: allocs.alloc(parse_rule_expr(&b[3], src, allocs, parse_a)?),
        },
        Construct(_, "Literal", b) => RuleExpr::Literal(parse_string(&b[0], src)?),
        Construct(_, "CharClass", b) => RuleExpr::CharClass(parse_charclass(&b[0], src, allocs)?),
        Construct(_, "SliceInput", b) => {
            RuleExpr::SliceInput(allocs.alloc(parse_rule_expr(&b[0], src, allocs, parse_a)?))
        }
        Construct(_, "PosLookahead", b) => {
            RuleExpr::PosLookahead(allocs.alloc(parse_rule_expr(&b[0], src, allocs, parse_a)?))
        }
        Construct(_, "NegLookahead", b) => {
            RuleExpr::NegLookahead(allocs.alloc(parse_rule_expr(&b[0], src, allocs, parse_a)?))
        }
        Construct(_, "This", _) => RuleExpr::This,
        Construct(_, "Next", _) => RuleExpr::Next,
        Construct(_, "Guid", _) => RuleExpr::Guid,
        Construct(_, "RunVar", b) => RuleExpr::RunVar(
            parse_identifier(&b[0], src)?,
            result_match! {
                create allocs.try_alloc_extend(b[1].iter_list().map(|sub| {
                    parse_rule_expr(sub, src, allocs, parse_a)
                }))?
            }?,
        ),
        Construct(_, "AtAdapt", b) => {
            RuleExpr::AtAdapt(parse_identifier(&b[0], src)?, parse_identifier(&b[1], src)?)
        }
        _ => return None,
    })
}

pub(crate) fn parse_identifier<'grm>(
    r: &ActionResult<'_, 'grm>,
    src: &'grm str,
) -> Option<&'grm str> {
    match r {
        Value(span) => Some(&src[*span]),
        // If the identifier of a block is a literal, it does not contain escaped chars
        Literal(s) => match s.to_cow() {
            Cow::Borrowed(s) => Some(s),
            Cow::Owned(_) => None,
        },
        _ => None,
    }
}

pub(crate) fn parse_string<'arn, 'grm>(
    r: &'arn ActionResult<'arn, 'grm>,
    src: &'grm str,
) -> Option<EscapedString<'grm>> {
    result_match! {
        match r => Value(span),
        create EscapedString::from_escaped(&src[*span])
    }
}

fn parse_string_char(r: &ActionResult<'_, '_>, src: &str) -> Option<char> {
    Some(match r {
        Value(span) => src[*span].chars().next().unwrap(),
        Literal(c) => c.chars().next().unwrap(),
        _ => return None,
    })
}

fn parse_charclass<'arn_out>(
    r: &ActionResult<'_, '_>,
    src: &str,
    allocs: Allocs<'arn_out>,
) -> Option<CharClass<'arn_out>> {
    result_match! {
        match r => Construct(_, "CharClass", b),
        create CharClass {
            neg: b[0].iter_list().next().is_some(),
            ranges: allocs.try_alloc_extend(b[1].iter_list().map(|p| result_match! {
                match p => Construct(_, "Range", pb),
                create (parse_string_char(&pb[0], src)?, parse_string_char(&pb[1], src)?)
            }))?
        }
    }
}

fn parse_option<'arn, 'grm, T>(
    r: &ActionResult<'arn, 'grm>,
    src: &str,
    sub: impl Fn(&ActionResult<'arn, 'grm>, &str) -> Option<T>,
) -> Option<Option<T>> {
    match r {
        Construct(_, "None", []) => Some(None),
        Construct(_, "Some", b) => Some(Some(sub(&b[0], src)?)),
        _ => None,
    }
}

fn parse_u64(r: &ActionResult<'_, '_>, src: &str) -> Option<u64> {
    match r {
        Literal(v) => v.parse().ok(),
        Value(span) => src[*span].parse().ok(),
        _ => None,
    }
}

pub fn parse_rule_action<'arn, 'grm>(
    r: &ActionResult<'_, 'grm>,
    src: &'grm str,
    allocs: Allocs<'arn>,
) -> Option<RuleAction<'arn, 'grm>> {
    Some(match r {
        Construct(_, "Construct", b) => RuleAction::Construct(
            parse_identifier(&b[0], src).unwrap(),
            parse_identifier(&b[1], src).unwrap(),
            result_match! {
                create allocs.try_alloc_extend(b[2].iter_list().map(|sub| parse_rule_action(sub, src, allocs)))?
            }?,
        ),
        Construct(_, "InputLiteral", b) => RuleAction::InputLiteral(parse_string(&b[0], src)?),
        Construct(_, "Name", b) => RuleAction::Name(parse_identifier(&b[0], src)?),
        _ => return None,
    })
}
