use crate::grammar::*;
use crate::parser_core::span::Span;
use crate::parser_sugar::action_result::ActionResult;
use crate::parser_sugar::action_result::ActionResult::*;

macro_rules! result_match {
    {match $e1:expr => $p1:pat_param, $(match $es:expr => $ps:pat_param,)* create $body:expr} => {
        match $e1 {
            $p1 => {
                result_match! { $(match $es => $ps,)* create $body }
            },
            _ => unreachable!(),
        }
    };
    {create $body:expr} => {
        $body
    };
}

pub fn parse_grammarfile<'grm>(r: &ActionResult<'grm>, src: &'grm str) -> GrammarFile<'grm> {
    result_match! {
        match r => Construct("GrammarFile", rules),
        match &rules[..] => [rules],
        match &**rules => List(rules),
        create GrammarFile {
            rules: rules.iter().map(|rule| parse_rule(rule, src)).collect(),
        }
    }
}

fn parse_rule<'grm>(r: &ActionResult<'grm>, src: &'grm str) -> Rule<'grm> {
    result_match! {
        match r => Construct("Rule", rule_body),
        match &rule_body[..] => [name, blocks],
        match &**blocks => List(blocks),
        create Rule {
            name: parse_identifier(name, src),
            blocks: blocks.iter().map(|block| parse_block(block, src)).collect(),
        }
    }
}

fn parse_block<'grm>(r: &ActionResult<'grm>, src: &'grm str) -> Block<'grm> {
    result_match! {
        match r => Construct("Block", b),
        match &b[..] => [name, cs],
        create Block(parse_identifier(name, src), parse_constructors(cs, src))
    }
}

fn parse_constructors<'grm>(
    r: &ActionResult<'grm>,
    src: &'grm str,
) -> Vec<AnnotatedRuleExpr<'grm>> {
    result_match! {
        match r => List(constructors),
        create constructors.iter().map(|c| parse_annotated_rule_expr(c, src)).collect()
    }
}

fn parse_annotated_rule_expr<'grm>(
    r: &ActionResult<'grm>,
    src: &'grm str,
) -> AnnotatedRuleExpr<'grm> {
    result_match! {
        match r => Construct("AnnotatedExpr", body),
        match &body[..] => [annots, e],
        match &**annots => List(annots),
        create AnnotatedRuleExpr(annots.iter().map(|annot| parse_rule_annotation(annot, src)).collect(), parse_rule_expr(e, src))
    }
}

fn parse_rule_annotation<'grm>(r: &ActionResult<'grm>, src: &'grm str) -> RuleAnnotation<'grm> {
    match r {
        Construct("Error", b) => RuleAnnotation::Error(parse_string(&b[0], src)),
        Construct("DisableLayout", _) => RuleAnnotation::DisableLayout,
        Construct("EnableLayout", _) => RuleAnnotation::EnableLayout,
        _ => unreachable!(),
    }
}

fn parse_rule_expr<'grm>(r: &ActionResult<'grm>, src: &'grm str) -> RuleExpr<'grm> {
    match r {
        Construct("Action", b) => RuleExpr::Action(
            Box::new(parse_rule_expr(&b[0], src)),
            parse_rule_action(&b[1], src),
        ),
        Construct("Choice", b) => RuleExpr::Choice(result_match! {
            match &*b[0] => List(subs),
            create subs.iter().map(|sub| parse_rule_expr(sub, src)).collect()
        }),
        Construct("Sequence", b) => RuleExpr::Sequence(result_match! {
            match &*b[0] => List(subs),
            create subs.iter().map(|sub| parse_rule_expr(sub, src)).collect()
        }),
        Construct("NameBind", b) => RuleExpr::NameBind(
            parse_identifier(&b[0], src),
            Box::new(parse_rule_expr(&b[1], src)),
        ),
        Construct("Repeat", b) => RuleExpr::Repeat {
            expr: Box::new(parse_rule_expr(&b[0], src)),
            min: parse_u64(&b[1], src),
            max: parse_option(&b[2], src, parse_u64),
            delim: Box::new(parse_rule_expr(&b[3], src)),
        },
        Construct("Literal", b) => RuleExpr::Literal(parse_string(&b[0], src)),
        Construct("CharClass", b) => RuleExpr::CharClass(parse_charclass(&b[0], src)),
        Construct("SliceInput", b) => RuleExpr::SliceInput(Box::new(parse_rule_expr(&b[0], src))),
        Construct("PosLookahead", b) => {
            RuleExpr::PosLookahead(Box::new(parse_rule_expr(&b[0], src)))
        }
        Construct("NegLookahead", b) => {
            RuleExpr::NegLookahead(Box::new(parse_rule_expr(&b[0], src)))
        }
        Construct("AtGrammar", _) => RuleExpr::AtGrammar,
        Construct("AtThis", _) => RuleExpr::AtThis,
        Construct("AtNext", _) => RuleExpr::AtNext,
        Construct("Rule", b) => RuleExpr::Rule(parse_identifier(&b[0], src)),
        Construct("AtAdapt", b) => {
            RuleExpr::AtAdapt(parse_rule_action(&b[0], src), parse_identifier(&b[1], src))
        }
        _ => unreachable!(),
    }
}

fn parse_rule_action<'grm>(r: &ActionResult<'grm>, src: &'grm str) -> RuleAction<'grm> {
    match r {
        Construct("Cons", b) => RuleAction::Cons(
            Box::new(parse_rule_action(&b[0], src)),
            Box::new(parse_rule_action(&b[1], src)),
        ),
        Construct("Nil", _) => RuleAction::Nil(),
        Construct("Construct", b) => RuleAction::Construct(
            parse_identifier(&b[0], src),
            result_match! {
                match &*b[1] => List(subs),
                create subs.iter().map(|sub| parse_rule_action(sub, src)).collect()
            },
        ),
        Construct("InputLiteral", b) => RuleAction::InputLiteral(parse_string(&b[0], src)),
        Construct("Name", b) => RuleAction::Name(parse_identifier(&b[0], src)),
        _ => unreachable!(),
    }
}

fn parse_identifier<'grm>(r: &ActionResult<'grm>, src: &'grm str) -> &'grm str {
    match r {
        Value(Span { start, end }) => &src[*start..*end],
        // If the identifier of a block is a literal, its always empty
        Literal(s) if s.chars().next().is_none() => "",
        _ => unreachable!(),
    }
}

fn parse_string<'grm>(r: &ActionResult<'grm>, src: &'grm str) -> EscapedString<'grm> {
    result_match! {
        match r => List(cs),
        create EscapedString::new(cs.iter().map(|c| parse_string_char(c, src)).collect())
    }
}

fn parse_string_char(r: &ActionResult, src: &str) -> char {
    match r {
        Value(Span { start, end }) => src[*start..*end].chars().next().unwrap(),
        Literal(c) => c.chars().next().unwrap(),
        _ => unreachable!(),
    }
}

fn parse_charclass(r: &ActionResult, src: &str) -> CharClass {
    result_match! {
        match r => Construct("CharClass", b),
        match &*b[0] => List(negate),
        match &*b[1] => List(ps),
        create CharClass {
            neg: !negate.is_empty(),
            ranges: ps.iter().map(|p| result_match! {
                match &**p => Construct("Range", pb),
                create (parse_string_char(&pb[0], src), parse_string_char(&pb[1], src))
            }).collect()
        }
    }
}

fn parse_option<T>(
    r: &ActionResult,
    src: &str,
    sub: impl Fn(&ActionResult, &str) -> T,
) -> Option<T> {
    match r {
        Construct("None", b) if b.is_empty() => None,
        Construct("Some", b) => Some(sub(&b[0], src)),
        _ => unreachable!(),
    }
}

fn parse_u64(r: &ActionResult, src: &str) -> u64 {
    match r {
        Literal(v) => v.parse().unwrap(),
        Value(Span { start, end }) => src[*start..*end].parse().unwrap(),
        _ => unreachable!(),
    }
}
