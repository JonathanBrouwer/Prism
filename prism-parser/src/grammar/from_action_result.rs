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

pub fn parse_grammarfile<'arn, 'grm, A>(
    r: &'arn ActionResult<'arn, 'grm>,
    src: &'grm str,
    parse_a: fn(&'arn ActionResult<'arn, 'grm>, src: &'grm str) -> Option<A>,
) -> Option<GrammarFile<'grm, A>> {
    result_match! {
        match r => Construct(_, "GrammarFile", rules),
        match &rules[..] => [rules],
        match rules.as_ref() => Construct(_, "List", rules),
        create GrammarFile {
            rules: rules.iter().map(|rule| parse_rule(rule, src, parse_a)).collect::<Option<Vec<_>>>()?,
        }
    }
}

fn parse_rule<'arn, 'grm, A>(
    r: &'arn ActionResult<'arn, 'grm>,
    src: &'grm str,
    parse_a: fn(&'arn ActionResult<'arn, 'grm>, src: &'grm str) -> Option<A>,
) -> Option<Rule<'grm, A>> {
    result_match! {
        match r => Construct(_, "Rule", rule_body),
        match &rule_body[..] => [name, args, blocks],
        match blocks.as_ref() => Construct(_, "List", blocks),
        match args.as_ref() => Construct(_, "List", args),
        create Rule {
            name: parse_identifier(name, src)?,
            args: args.iter().map(|n| parse_identifier(n, src)).collect::<Option<Vec<_>>>()?,
            blocks: blocks.iter().map(|block| parse_block(block, src, parse_a)).collect::<Option<Vec<_>>>()?,
        }
    }
}

fn parse_block<'arn, 'grm, A>(
    r: &'arn ActionResult<'arn, 'grm>,
    src: &'grm str,
    parse_a: fn(&'arn ActionResult<'arn, 'grm>, src: &'grm str) -> Option<A>,
) -> Option<Block<'grm, A>> {
    result_match! {
        match r => Construct(_, "Block", b),
        match &b[..] => [name, cs],
        create crate::grammar::Block(parse_identifier(name, src)?, parse_constructors(cs, src, parse_a)?)
    }
}

fn parse_constructors<'arn, 'grm, A>(
    r: &'arn ActionResult<'arn, 'grm>,
    src: &'grm str,
    parse_a: fn(&'arn ActionResult<'arn, 'grm>, src: &'grm str) -> Option<A>,
) -> Option<Vec<AnnotatedRuleExpr<'grm, A>>> {
    result_match! {
        match r => Construct(_, "List", constructors),
        create constructors.iter().map(|c| parse_annotated_rule_expr(c, src, parse_a)).collect::<Option<Vec<_>>>()?
    }
}

fn parse_annotated_rule_expr<'arn, 'grm, A>(
    r: &'arn ActionResult<'arn, 'grm>,
    src: &'grm str,
    parse_a: fn(&'arn ActionResult<'arn, 'grm>, src: &'grm str) -> Option<A>,
) -> Option<AnnotatedRuleExpr<'grm, A>> {
    result_match! {
        match r => Construct(_, "AnnotatedExpr", body),
        match &body[..] => [annots, e],
        match annots.as_ref() => Construct(_, "List", annots),
        create crate::grammar::AnnotatedRuleExpr(annots.iter().map(|annot| parse_rule_annotation(annot, src)).collect::<Option<Vec<_>>>()?, parse_rule_expr(e, src, parse_a)?)
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

fn parse_rule_expr<'arn, 'grm, A>(
    r: &'arn ActionResult<'arn, 'grm>,
    src: &'grm str,
    parse_a: fn(&'arn ActionResult<'arn, 'grm>, src: &'grm str) -> Option<A>,
) -> Option<RuleExpr<'grm, A>> {
    Some(match r {
        Construct(_, "Action", b) => RuleExpr::Action(
            Box::new(parse_rule_expr(&b[0], src, parse_a)?),
            parse_a(&b[1], src)?,
        ),
        Construct(_, "Choice", b) => RuleExpr::Choice(result_match! {
            match &b[0].as_ref() => Construct(_, "List", subs),
            create subs.iter().map(|sub| parse_rule_expr(sub, src, parse_a)).collect::<Option<Vec<_>>>()?
        }?),
        Construct(_, "Sequence", b) => RuleExpr::Sequence(result_match! {
            match &b[0].as_ref() => Construct(_, "List", subs),
            create subs.iter().map(|sub| parse_rule_expr(sub, src, parse_a)).collect::<Option<Vec<_>>>()?
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
        Construct(_, "AtThis", _) => RuleExpr::AtThis,
        Construct(_, "AtNext", _) => RuleExpr::AtNext,
        Construct(_, "Rule", b) => RuleExpr::Rule(
            parse_identifier(&b[0], src)?,
            result_match! {
                match &b[1].as_ref() => Construct(_, "List", args),
                create args.iter().map(|sub| parse_a(sub, src)).collect::<Option<Vec<_>>>()?
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
        match &b[0].as_ref() => Construct(_, "List", negate),
        match &b[1].as_ref() => Construct(_, "List", ps),
        create CharClass {
            neg: !negate.is_empty(),
            ranges: ps.iter().map(|p| result_match! {
                match p.as_ref() => Construct(_, "Range", pb),
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
        Construct(_, "None", b) if b.is_empty() => Some(None),
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
