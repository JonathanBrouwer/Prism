use crate::peg_parser::peg_parser::{TokenType, Token, PegRule};
use std::collections::HashMap;
use crate::peg_parser::peg_parser::PegRule::Sequence;

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum NicePegRule<'a, TT: TokenType, T: Token<TT>> {
    LiteralExact(T),
    LiteralBind(TT),

    Sequence(Vec<NicePegRule<'a, TT, T>>),
    ChooseFirst(Vec<NicePegRule<'a, TT, T>>),

    // Repeat(Box<NicePegRule<'a, TT, T>>, Option<usize>, Option<usize>),
    // Option(Box<NicePegRule<'a, TT, T>>),

    LookaheadPositive(Box<NicePegRule<'a, TT, T>>),
    LookaheadNegative(Box<NicePegRule<'a, TT, T>>),

    Rule(&'a str)
}

fn handle_rule<'a, TT: TokenType, T: Token<TT>>(name: &'a str, rule: NicePegRule<'a, TT, T>, at: Option<usize>, new_rules: &mut Vec<PegRule<TT, T>>, rule_index: &HashMap<&'a str, usize>) -> usize {
    let v = match rule {
        NicePegRule::LiteralExact(v) =>
            PegRule::LiteralExact(v),
        NicePegRule::LiteralBind(v) =>
            PegRule::LiteralBind(v),
        NicePegRule::Sequence(vs) =>
            PegRule::Sequence(vs.into_iter().map(|sub_rule| handle_rule(name, sub_rule, None, new_rules, rule_index)).collect()),
        NicePegRule::ChooseFirst(vs) =>
            PegRule::ChooseFirst(vs.into_iter().map(|sub_rule| handle_rule(name, sub_rule, None, new_rules, rule_index)).collect()),
        // NicePegRule::Repeat(v, min, max) =>
        //     PegRule::Repeat(handle_rule(name, *v, None, new_rules, rule_index), min, max),
        // NicePegRule::Option(v) =>
        //     PegRule::Option(handle_rule(name, *v, None, new_rules, rule_index)),
        NicePegRule::LookaheadPositive(v) =>
            PegRule::LookaheadPositive(handle_rule(name, *v, None, new_rules, rule_index)),
        NicePegRule::LookaheadNegative(v) =>
            PegRule::LookaheadNegative(handle_rule(name, *v, None, new_rules, rule_index)),
        NicePegRule::Rule(rule) => return *rule_index.get(rule).unwrap()
    };
    match at {
        None => {
            new_rules.push(v);
            new_rules.len() - 1
        },
        Some(at) => {
            new_rules[at] = v;
            at
        }
    }
}

pub fn nice_rules_to_peg<'a, TT: TokenType, T: Token<TT>>(rules: HashMap<&'a str, NicePegRule<'a, TT, T>>, start: &'a str) -> (Vec<PegRule<TT, T>>, usize) {
    let mut new_rules: Vec<PegRule<TT, T>> = Vec::with_capacity(rules.len() * 20);
    let mut rule_index : HashMap<&'a str, usize> = HashMap::new();

    for (i, (&name, _)) in rules.iter().enumerate() {
        new_rules.push(Sequence(vec![]));
        rule_index.insert(name, i);
    }

    for (name, rule) in rules.into_iter() {
        handle_rule(name, rule, Some(*rule_index.get(name).unwrap()), &mut new_rules, &rule_index);
    }

    (new_rules, *rule_index.get(start).unwrap())
}