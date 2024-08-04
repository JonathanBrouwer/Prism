use crate::grammar::escaped_string::EscapedString;
use crate::grammar::{AnnotatedRuleExpr, Block, GrammarFile, Rule, RuleExpr};
use crate::grammar::{CharClass, RuleAnnotation};
use crate::rule_action::action_result::ActionResult;
use crate::rule_action::action_result::ActionResult::*;

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

pub fn parse_grammarfile<'arn, 'grm, Action>(
    r: &'arn ActionResult<'arn, 'grm>,
    src: &'grm str,
    parse_a: fn(&'arn ActionResult<'arn, 'grm>, src: &'grm str) -> Option<Action>,
) -> Option<GrammarFile<'grm, Action>> {
    result_match! {
        match r => Construct(_, "GrammarFile", rules),
        match &rules[..] => [rules],
        create GrammarFile {
            rules: rules.iter_list().map(|rule| parse_rule(rule, src, parse_a)).collect::<Option<Vec<_>>>()?,
        }
    }
}

fn parse_rule<'arn, 'grm, Action>(
    r: &'arn ActionResult<'arn, 'grm>,
    src: &'grm str,
    parse_a: fn(&'arn ActionResult<'arn, 'grm>, src: &'grm str) -> Option<Action>,
) -> Option<Rule<'grm, Action>> {
    result_match! {
        match r => Construct(_, "Rule", rule_body),
        match &rule_body[..] => [name, args, blocks],
        create Rule {
            name: parse_identifier(name, src)?,
            args: args.iter_list().map(|n| parse_identifier(n, src)).collect::<Option<Vec<_>>>()?,
            blocks: blocks.iter_list().map(|block| parse_block(block, src, parse_a)).collect::<Option<Vec<_>>>()?,
        }
    }
}

fn parse_block<'arn, 'grm, Action>(
    r: &'arn ActionResult<'arn, 'grm>,
    src: &'grm str,
    parse_a: fn(&'arn ActionResult<'arn, 'grm>, src: &'grm str) -> Option<Action>,
) -> Option<Block<'grm, Action>> {
    result_match! {
        match r => Construct(_, "Block", b),
        match &b[..] => [name, cs],
        create Block(parse_identifier(name, src)?, parse_constructors(cs, src, parse_a)?)
    }
}

fn parse_constructors<'arn, 'grm, Action>(
    r: &'arn ActionResult<'arn, 'grm>,
    src: &'grm str,
    parse_a: fn(&'arn ActionResult<'arn, 'grm>, src: &'grm str) -> Option<Action>,
) -> Option<Vec<AnnotatedRuleExpr<'grm, Action>>> {
    result_match! {
        create r.iter_list().map(|c| parse_annotated_rule_expr(c, src, parse_a)).collect::<Option<Vec<_>>>()?
    }
}

fn parse_annotated_rule_expr<'arn, 'grm, Action>(
    r: &'arn ActionResult<'arn, 'grm>,
    src: &'grm str,
    parse_a: fn(&'arn ActionResult<'arn, 'grm>, src: &'grm str) -> Option<Action>,
) -> Option<AnnotatedRuleExpr<'grm, Action>> {
    result_match! {
        match r => Construct(_, "AnnotatedExpr", body),
        match &body[..] => [annots, e],
        create AnnotatedRuleExpr(annots.iter_list().map(|annot| parse_rule_annotation(annot, src)).collect::<Option<Vec<_>>>()?, parse_rule_expr(e, src, parse_a)?)
    }
}

fn parse_rule_annotation<'arn, 'grm>(
    r: &'arn ActionResult<'arn, 'grm>,
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

fn parse_rule_expr<'arn, 'grm, Action>(
    r: &'arn ActionResult<'arn, 'grm>,
    src: &'grm str,
    parse_a: fn(&'arn ActionResult<'arn, 'grm>, src: &'grm str) -> Option<Action>,
) -> Option<RuleExpr<'grm, Action>> {
    Some(match r {
        Construct(_, "Action", b) => RuleExpr::Action(
            Box::new(parse_rule_expr(&b[0], src, parse_a)?),
            parse_a(&b[1], src)?,
        ),
        Construct(_, "Choice", b) => RuleExpr::Choice(result_match! {
            create b[0].iter_list().map(|sub| parse_rule_expr(sub, src, parse_a)).collect::<Option<Vec<_>>>()?
        }?),
        Construct(_, "Sequence", b) => RuleExpr::Sequence(result_match! {
            create b[0].iter_list().map(|sub| parse_rule_expr(sub, src, parse_a)).collect::<Option<Vec<_>>>()?
        }?),
        Construct(_, "NameBind", b) => RuleExpr::NameBind(
            parse_identifier(&b[0], src)?,
            Box::new(parse_rule_expr(&b[1], src, parse_a)?),
        ),
        Construct(_, "Repeat", b) => RuleExpr::Repeat {
            expr: Box::new(parse_rule_expr(&b[0], src, parse_a)?),
            min: parse_u64(&b[1], src)?,
            max: parse_option(&b[2], src, parse_u64)?,
            delim: Box::new(parse_rule_expr(&b[3], src, parse_a)?),
        },
        Construct(_, "Literal", b) => RuleExpr::Literal(parse_string(&b[0], src)?),
        Construct(_, "CharClass", b) => RuleExpr::CharClass(parse_charclass(&b[0], src)?),
        Construct(_, "SliceInput", b) => {
            RuleExpr::SliceInput(Box::new(parse_rule_expr(&b[0], src, parse_a)?))
        }
        Construct(_, "PosLookahead", b) => {
            RuleExpr::PosLookahead(Box::new(parse_rule_expr(&b[0], src, parse_a)?))
        }
        Construct(_, "NegLookahead", b) => {
            RuleExpr::NegLookahead(Box::new(parse_rule_expr(&b[0], src, parse_a)?))
        }
        Construct(_, "This", _) => RuleExpr::This,
        Construct(_, "Next", _) => RuleExpr::Next,
        Construct(_, "Guid", _) => RuleExpr::Guid,
        Construct(_, "RunVar", b) => RuleExpr::RunVar(
            parse_identifier(&b[0], src)?,
            result_match! {
                create b[1].iter_list().map(|sub| {
                    parse_rule_expr(sub, src, parse_a)
                }).collect::<Option<Vec<_>>>()?
            }?,
        ),
        Construct(_, "AtAdapt", b) => {
            RuleExpr::AtAdapt(parse_a(&b[0], src)?, parse_identifier(&b[1], src)?)
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
        // If the identifier of a block is a literal, its always empty
        Literal(s) if s.chars().next().is_none() => Some(""),
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

fn parse_charclass(r: &ActionResult<'_, '_>, src: &str) -> Option<CharClass> {
    result_match! {
        match r => Construct(_, "CharClass", b),
        create CharClass {
            neg: b[0].iter_list().next().is_some(),
            ranges: b[1].iter_list().map(|p| result_match! {
                match p => Construct(_, "Range", pb),
                create (parse_string_char(&pb[0], src)?, parse_string_char(&pb[1], src)?)
            }).collect::<Option<Vec<_>>>()?
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
