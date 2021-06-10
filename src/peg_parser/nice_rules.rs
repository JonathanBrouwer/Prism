use crate::peg_parser::peg_parser::{PegRule};
use std::collections::HashMap;
use crate::peg_parser::peg_parser::PegRule::Sequence;
use crate::peg_parser::parser_token::*;

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum NicePegRule<'a, TT: TokenType, TV: TokenValue> {
    LiteralExact(TV),
    LiteralBind(TT),

    Sequence(Vec<NicePegRule<'a, TT, TV>>),
    ChooseFirst(Vec<NicePegRule<'a, TT, TV>>),

    LookaheadPositive(Box<NicePegRule<'a, TT, TV>>),
    LookaheadNegative(Box<NicePegRule<'a, TT, TV>>),

    Rule(&'a str)
}

fn handle_rule<'a, TT: TokenType, TV: TokenValue>(name: &'a str, rule: NicePegRule<'a, TT, TV>, at: Option<usize>, new_rules: &mut Vec<PegRule<TT, TV>>, rule_index: &HashMap<&'a str, usize>) -> usize {
    let v = match rule {
        NicePegRule::LiteralExact(v) =>
            PegRule::LiteralExact(v),
        NicePegRule::LiteralBind(v) =>
            PegRule::LiteralBind(v),
        NicePegRule::Sequence(vs) =>
            PegRule::Sequence(vs.into_iter().map(|sub_rule| handle_rule(name, sub_rule, None, new_rules, rule_index)).collect()),
        NicePegRule::ChooseFirst(vs) =>
            PegRule::ChooseFirst(vs.into_iter().map(|sub_rule| handle_rule(name, sub_rule, None, new_rules, rule_index)).collect()),
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

pub fn nice_rules_to_peg<'a, TT: TokenType, TV: TokenValue>(rules: HashMap<&'a str, NicePegRule<'a, TT, TV>>, start: &'a str) -> (Vec<PegRule<TT, TV>>, usize) {
    let mut new_rules: Vec<PegRule<TT, TV>> = Vec::with_capacity(rules.len() * 20);
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