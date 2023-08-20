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
        match r => Construct(_, "GrammarFile", rules),
        match &rules[..] => [rules],
        match &**rules => Construct(_, "List", rules),
        create GrammarFile {
            rules: rules.iter().map(|rule| parse_rule(rule, src)).collect::<Option<Vec<_>>>()?,
        }
    }
}

fn parse_rule<'grm>(r: &ActionResult<'grm>, src: &'grm str) -> Option<Rule<'grm>> {
    result_match! {
        match r => Construct(_, "Rule", rule_body),
        match &rule_body[..] => [name, blocks],
        match &**blocks => Construct(_, "List", blocks),
        create Rule {
            name: parse_identifier(name, src)?,
            args: Vec::new(),
            blocks: blocks.iter().map(|block| parse_block(block, src)).collect::<Option<Vec<_>>>()?,
        }
    }
}

fn parse_block<'grm>(r: &ActionResult<'grm>, src: &'grm str) -> Option<Block<'grm>> {
    result_match! {
        match r => Construct(_, "Block", b),
        match &b[..] => [name, cs],
        create Block(parse_identifier(name, src)?, parse_constructors(cs, src)?)
    }
}

fn parse_constructors<'grm>(
    r: &ActionResult<'grm>,
    src: &'grm str,
) -> Option<Vec<AnnotatedRuleExpr<'grm>>> {
    result_match! {
        match r => Construct(_, "List", constructors),
        create constructors.iter().map(|c| parse_annotated_rule_expr(c, src)).collect::<Option<Vec<_>>>()?
    }
}

fn parse_annotated_rule_expr<'grm>(
    r: &ActionResult<'grm>,
    src: &'grm str,
) -> Option<AnnotatedRuleExpr<'grm>> {
    result_match! {
        match r => Construct(_, "AnnotatedExpr", body),
        match &body[..] => [annots, e],
        match &**annots => Construct(_, "List", annots),
        create AnnotatedRuleExpr(annots.iter().map(|annot| parse_rule_annotation(annot, src)).collect::<Option<Vec<_>>>()?, parse_rule_expr(e, src)?)
    }
}

fn parse_rule_annotation<'grm>(
    r: &ActionResult<'grm>,
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

fn parse_rule_expr<'grm>(r: &ActionResult<'grm>, src: &'grm str) -> Option<RuleExpr<'grm>> {
    Some(match r {
        Construct(_, "Action", b) => RuleExpr::Action(
            Box::new(parse_rule_expr(&b[0], src)?),
            parse_rule_action(&b[1], src)?,
        ),
        Construct(_, "Choice", b) => RuleExpr::Choice(result_match! {
            match &*b[0] => Construct(_, "List", subs),
            create subs.iter().map(|sub| parse_rule_expr(sub, src)).collect::<Option<Vec<_>>>()?
        }?),
        Construct(_, "Sequence", b) => RuleExpr::Sequence(result_match! {
            match &*b[0] => Construct(_, "List", subs),
            create subs.iter().map(|sub| parse_rule_expr(sub, src)).collect::<Option<Vec<_>>>()?
        }?),
        Construct(_, "NameBind", b) => RuleExpr::NameBind(
            parse_identifier(&b[0], src)?,
            Box::new(parse_rule_expr(&b[1], src)?),
        ),
        Construct(_, "Repeat", b) => RuleExpr::Repeat {
            expr: Box::new(parse_rule_expr(&b[0], src)?),
            min: parse_u64(&b[1], src)?,
            max: parse_option(&b[2], src, parse_u64)?,
            delim: Box::new(parse_rule_expr(&b[3], src)?),
        },
        Construct(_, "Literal", b) => RuleExpr::Literal(parse_string(&b[0], src)?),
        Construct(_, "CharClass", b) => RuleExpr::CharClass(parse_charclass(&b[0], src)?),
        Construct(_, "SliceInput", b) => RuleExpr::SliceInput(Box::new(parse_rule_expr(&b[0], src)?)),
        Construct(_, "PosLookahead", b) => {
            RuleExpr::PosLookahead(Box::new(parse_rule_expr(&b[0], src)?))
        }
        Construct(_, "NegLookahead", b) => {
            RuleExpr::NegLookahead(Box::new(parse_rule_expr(&b[0], src)?))
        }
        Construct(_, "AtGrammar", _) => RuleExpr::AtGrammar,
        Construct(_, "AtThis", _) => RuleExpr::AtThis,
        Construct(_, "AtNext", _) => RuleExpr::AtNext,
        Construct(_, "Rule", b) => RuleExpr::Rule(parse_identifier(&b[0], src)?),
        Construct(_, "AtAdapt", b) => RuleExpr::AtAdapt(
            parse_rule_action(&b[0], src)?,
            parse_identifier(&b[1], src)?,
        ),
        _ => return None,
    })
}

fn parse_rule_action<'grm>(r: &ActionResult<'grm>, src: &'grm str) -> Option<RuleAction<'grm>> {
    Some(match r {
        Construct(_, "Cons", b) => RuleAction::Cons(
            Box::new(parse_rule_action(&b[0], src)?),
            Box::new(parse_rule_action(&b[1], src)?),
        ),
        Construct(_, "Nil", _) => RuleAction::Nil(),
        Construct(_, "Construct", b) => RuleAction::Construct(
            parse_identifier(&b[0], src)?,
            result_match! {
                match &*b[1] => Construct(_, "List", subs),
                create subs.iter().map(|sub| parse_rule_action(sub, src)).collect::<Option<Vec<_>>>()?
            }?,
        ),
        Construct(_, "InputLiteral", b) => RuleAction::InputLiteral(parse_string(&b[0], src)?),
        Construct(_, "Name", b) => RuleAction::Name(parse_identifier(&b[0], src)?),
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
        match r => Construct(_, "CharClass", b),
        match &*b[0] => Construct(_, "List", negate),
        match &*b[1] => Construct(_, "List", ps),
        create CharClass {
            neg: !negate.is_empty(),
            ranges: ps.iter().map(|p| result_match! {
                match &**p => Construct(_, "Range", pb),
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
        Construct(_, "None", b) if b.is_empty() => Some(None),
        Construct(_, "Some", b) => Some(Some(sub(&b[0], src)?)),
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
