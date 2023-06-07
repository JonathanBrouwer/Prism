use crate::grammar::action_result::ActionResult;
use crate::grammar::action_result::ActionResult::*;
use crate::grammar::escaped_string::EscapedString;
use crate::grammar::grammar::*;

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

pub fn parse_grammarfile<'grm>(
    r: &ActionResult<'grm>,
    src: &'grm str,
) -> Option<GrammarFile<'grm>> {
    result_match! {
        match r => Construct("GrammarFile", rules),
        match &rules[..] => [rules],
        match &**rules => Construct("List", rules),
        create GrammarFile {
            rules: rules.iter().map(|rule| parse_rule(rule, src)).collect::<Option<Vec<_>>>()?,
        }
    }
}

fn parse_rule<'grm>(r: &ActionResult<'grm>, src: &'grm str) -> Option<Rule<'grm>> {
    result_match! {
        match r => Construct("Rule", rule_body),
        match &rule_body[..] => [name, blocks],
        match &**blocks => Construct("List", blocks),
        create Rule {
            name: parse_identifier(name, src)?,
            blocks: blocks.iter().map(|block| parse_block(block, src)).collect::<Option<Vec<_>>>()?,
        }
    }
}

fn parse_block<'grm>(r: &ActionResult<'grm>, src: &'grm str) -> Option<Block<'grm>> {
    result_match! {
        match r => Construct("Block", b),
        match &b[..] => [name, cs],
        create Block(parse_identifier(name, src)?, parse_constructors(cs, src)?)
    }
}

fn parse_constructors<'grm>(
    r: &ActionResult<'grm>,
    src: &'grm str,
) -> Option<Vec<AnnotatedRuleExpr<'grm>>> {
    result_match! {
        match r => Construct("List", constructors),
        create constructors.iter().map(|c| parse_annotated_rule_expr(c, src)).collect::<Option<Vec<_>>>()?
    }
}

fn parse_annotated_rule_expr<'grm>(
    r: &ActionResult<'grm>,
    src: &'grm str,
) -> Option<AnnotatedRuleExpr<'grm>> {
    result_match! {
        match r => Construct("AnnotatedExpr", body),
        match &body[..] => [annots, e],
        match &**annots => Construct("List", annots),
        create AnnotatedRuleExpr(annots.iter().map(|annot| parse_rule_annotation(annot, src)).collect::<Option<Vec<_>>>()?, parse_rule_expr(e, src)?)
    }
}

fn parse_rule_annotation<'grm>(
    r: &ActionResult<'grm>,
    src: &'grm str,
) -> Option<RuleAnnotation<'grm>> {
    Some(match r {
        Construct("Error", b) => RuleAnnotation::Error(parse_string(&b[0], src)?),
        Construct("DisableLayout", _) => RuleAnnotation::DisableLayout,
        Construct("EnableLayout", _) => RuleAnnotation::EnableLayout,
        Construct("DisableRecovery", _) => RuleAnnotation::DisableRecovery,
        Construct("EnableRecovery", _) => RuleAnnotation::EnableRecovery,
        _ => return None,
    })
}

fn parse_rule_expr<'grm>(r: &ActionResult<'grm>, src: &'grm str) -> Option<RuleExpr<'grm>> {
    Some(match r {
        Construct("Action", b) => RuleExpr::Action(
            Box::new(parse_rule_expr(&b[0], src)?),
            parse_rule_action(&b[1], src)?,
        ),
        Construct("Choice", b) => RuleExpr::Choice(result_match! {
            match &*b[0] => Construct("List", subs),
            create subs.iter().map(|sub| parse_rule_expr(sub, src)).collect::<Option<Vec<_>>>()?
        }?),
        Construct("Sequence", b) => RuleExpr::Sequence(result_match! {
            match &*b[0] => Construct("List", subs),
            create subs.iter().map(|sub| parse_rule_expr(sub, src)).collect::<Option<Vec<_>>>()?
        }?),
        Construct("NameBind", b) => RuleExpr::NameBind(
            parse_identifier(&b[0], src)?,
            Box::new(parse_rule_expr(&b[1], src)?),
        ),
        Construct("Repeat", b) => RuleExpr::Repeat {
            expr: Box::new(parse_rule_expr(&b[0], src)?),
            min: parse_u64(&b[1], src)?,
            max: parse_option(&b[2], src, parse_u64)?,
            delim: Box::new(parse_rule_expr(&b[3], src)?),
        },
        Construct("Literal", b) => RuleExpr::Literal(parse_string(&b[0], src)?),
        Construct("CharClass", b) => RuleExpr::CharClass(parse_charclass(&b[0], src)?),
        Construct("SliceInput", b) => RuleExpr::SliceInput(Box::new(parse_rule_expr(&b[0], src)?)),
        Construct("PosLookahead", b) => {
            RuleExpr::PosLookahead(Box::new(parse_rule_expr(&b[0], src)?))
        }
        Construct("NegLookahead", b) => {
            RuleExpr::NegLookahead(Box::new(parse_rule_expr(&b[0], src)?))
        }
        Construct("AtGrammar", _) => RuleExpr::AtGrammar,
        Construct("AtThis", _) => RuleExpr::AtThis,
        Construct("AtNext", _) => RuleExpr::AtNext,
        Construct("Rule", b) => RuleExpr::Rule(parse_identifier(&b[0], src)?),
        Construct("AtAdapt", b) => RuleExpr::AtAdapt(
            parse_rule_action(&b[0], src)?,
            parse_identifier(&b[1], src)?,
        ),
        _ => return None,
    })
}

fn parse_rule_action<'grm>(r: &ActionResult<'grm>, src: &'grm str) -> Option<RuleAction<'grm>> {
    Some(match r {
        Construct("Cons", b) => RuleAction::Cons(
            Box::new(parse_rule_action(&b[0], src)?),
            Box::new(parse_rule_action(&b[1], src)?),
        ),
        Construct("Nil", _) => RuleAction::Nil(),
        Construct("Construct", b) => RuleAction::Construct(
            parse_identifier(&b[0], src)?,
            result_match! {
                match &*b[1] => Construct("List", subs),
                create subs.iter().map(|sub| parse_rule_action(sub, src)).collect::<Option<Vec<_>>>()?
            }?,
        ),
        Construct("InputLiteral", b) => RuleAction::InputLiteral(parse_string(&b[0], src)?),
        Construct("Name", b) => RuleAction::Name(parse_identifier(&b[0], src)?),
        _ => return None,
    })
}

fn parse_identifier<'grm>(r: &ActionResult<'grm>, src: &'grm str) -> Option<&'grm str> {
    match r {
        Value(span) => Some(&src[*span]),
        // If the identifier of a block is a literal, its always empty
        Literal(s) if s.chars().next().is_none() => Some(""),
        _ => None,
    }
}

fn parse_string<'grm>(r: &ActionResult<'grm>, src: &'grm str) -> Option<EscapedString<'grm>> {
    result_match! {
        match r => Value(span),
        create EscapedString::from_escaped(&src[*span])
    }
}

fn parse_string_char(r: &ActionResult, src: &str) -> Option<char> {
    Some(match r {
        Value(span) => src[*span].chars().next().unwrap(),
        Literal(c) => c.chars().next().unwrap(),
        _ => return None,
    })
}

fn parse_charclass(r: &ActionResult, src: &str) -> Option<CharClass> {
    result_match! {
        match r => Construct("CharClass", b),
        match &*b[0] => Construct("List", negate),
        match &*b[1] => Construct("List", ps),
        create CharClass {
            neg: !negate.is_empty(),
            ranges: ps.iter().map(|p| result_match! {
                match &**p => Construct("Range", pb),
                create (parse_string_char(&pb[0], src)?, parse_string_char(&pb[1], src)?)
            }).collect::<Option<Vec<_>>>()?
        }
    }
}

fn parse_option<T>(
    r: &ActionResult,
    src: &str,
    sub: impl Fn(&ActionResult, &str) -> Option<T>,
) -> Option<Option<T>> {
    match r {
        Construct("None", b) if b.is_empty() => Some(None),
        Construct("Some", b) => Some(Some(sub(&b[0], src)?)),
        _ => None,
    }
}

fn parse_u64(r: &ActionResult, src: &str) -> Option<u64> {
    match r {
        Literal(v) => v.parse().ok(),
        Value(span) => src[*span].parse().ok(),
        _ => None,
    }
}
