use crate::grammar::{RuleAction, RuleBody};
use crate::parser::parser_core::ParserState;
use crate::parser::parser_result::ParseResult;
use itertools::Itertools;
use std::collections::HashMap;

pub type PR<'grm> = (HashMap<&'grm str, ActionResult<'grm>>, ActionResult<'grm>);

#[derive(Clone)]
pub enum ActionResult<'grm> {
    Value((usize, usize)),
    Literal(&'grm str),
    Construct(&'grm str, Vec<ActionResult<'grm>>),
    Error,
}

impl<'grm, 'src> ParserState<'grm, 'src> {
    pub fn parse_rule(
        &mut self,
        pos: usize,
        rules: &HashMap<&'grm str, RuleBody<'grm>>,
        rule: &'grm str,
    ) -> ParseResult<PR> {
        self.parse_expr(pos, rules, &rules.get(rule).unwrap())
    }

    pub fn parse_expr(
        &mut self,
        pos: usize,
        rules: &HashMap<&'grm str, RuleBody<'grm>>,
        expr: &RuleBody<'grm>,
    ) -> ParseResult<PR<'grm>> {
        match expr {
            RuleBody::Rule(_) => {
                todo!()
            }
            RuleBody::CharClass(cc) => {
                let result = self.parse_charclass(pos, cc);
                let new_pos = result.pos;
                result.map(|_| (HashMap::new(), ActionResult::Value((pos, new_pos))))
            }
            RuleBody::Literal(_) => {
                todo!()
            }
            RuleBody::Repeat { .. } => {
                todo!()
            }
            RuleBody::Sequence(subs) => {
                let mut state = ParseResult::new_ok((HashMap::new(), ActionResult::Error), pos);
                for sub in subs {
                    let res: ParseResult<(PR, PR)> =
                        self.parse_sequence(state, |s, p| s.parse_expr(p, rules, sub));
                    state = res.map(|(mut l, r)| {
                        for (k, v) in r.0.into_iter() {
                            l.0.insert(k, v);
                        }
                        l
                    });
                }
                state
            }
            RuleBody::Choice(_) => {
                todo!()
            }
            RuleBody::NameBind(name, sub) => {
                let res = self.parse_expr(pos, rules, sub);
                res.map(|mut res| {
                    res.0.insert(name, res.1.clone());
                    res
                })
            }
            RuleBody::Action(sub, action) => {
                let res = self.parse_expr(pos, rules, sub);
                res.map(|mut res| {
                    res.1 = apply_action(action, &res.0);
                    res
                })
            }
        }
    }
}

fn apply_action<'grm>(
    rule: &RuleAction<'grm>,
    map: &HashMap<&str, ActionResult<'grm>>,
) -> ActionResult<'grm> {
    match rule {
        RuleAction::Name(name) => {
            if let Some(v) = map.get(name) {
                v.clone()
            } else {
                ActionResult::Error
            }
        }
        RuleAction::InputLiteral(lit) => ActionResult::Literal(lit),
        RuleAction::Construct(name, args) => {
            let args_vals = args.iter().map(|a| apply_action(a, map)).collect_vec();
            ActionResult::Construct(name, args_vals)
        }
    }
}
