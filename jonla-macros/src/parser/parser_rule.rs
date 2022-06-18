use crate::grammar::{CharClass, RuleAction, RuleBody};
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
        rule: &str,
    ) -> ParseResult<PR<'grm>> {
        self.parse_expr(pos, rules, &rules.get(rule).unwrap())
    }

    pub fn parse_expr(
        &mut self,
        pos: usize,
        rules: &HashMap<&'grm str, RuleBody<'grm>>,
        expr: &RuleBody<'grm>,
    ) -> ParseResult<PR<'grm>> {
        match expr {
            RuleBody::Rule(rule) => self.parse_rule(pos, rules, rule),
            RuleBody::CharClass(cc) => {
                let result = self.parse_charclass(pos, cc);
                let new_pos = result.pos;
                result.map(|_| (HashMap::new(), ActionResult::Value((pos, new_pos))))
            }
            RuleBody::Literal(literal) => {
                let mut state = ParseResult::new_ok((), pos);
                for char in literal.chars() {
                    state = self
                        .parse_sequence(state, |s, p| {
                            s.parse_charclass(
                                p,
                                &CharClass {
                                    ranges: vec![(char, char)],
                                },
                            )
                        })
                        .map(|_| ());
                }
                let new_pos = state.pos;
                state.map(|_| (HashMap::new(), ActionResult::Value((pos, new_pos))))
            }
            RuleBody::Repeat {
                expr,
                min,
                max,
                delim,
                trailing_delim,
            } => {
                let mut state = ParseResult::new_ok((HashMap::new(), ()), pos);
                let mut results = vec![];

                for _ in 0..*min {
                    //Parse expr
                    let res = self.parse_sequence(state, |s, p| s.parse_expr(p, rules, expr));
                    state = res.map(|(l, r)| {
                        results.push(r.1);
                        l
                    })
                }
                while state.is_ok() {
                    //Parse expr
                    let res = self.parse_sequence(state.clone(), |s, p| s.parse_expr(p, rules, expr));
                    if !res.is_ok() { break; }
                    state = res.map(|(l, r)| {
                        results.push(r.1);
                        l
                    })
                }

                state.map(|(map, _)| (map, ActionResult::Error))
            }
            RuleBody::Sequence(subs) => {
                let mut state = ParseResult::new_ok((HashMap::new(), ()), pos);
                for sub in subs {
                    let res = self.parse_sequence(state, |s, p| s.parse_expr(p, rules, sub));
                    state = res.map(|(mut l, r)| {
                        for (k, v) in r.0.into_iter() {
                            l.0.insert(k, v);
                        }
                        l
                    });
                }
                state.map(|(map, _)| (map, ActionResult::Error))
            }
            RuleBody::Choice(subs) => {
                let mut state = ParseResult::new_err(pos);
                for sub in subs {
                    state = self.parse_choice(pos, state, |s, p| s.parse_expr(p, rules, sub));
                }
                state
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
            RuleBody::SliceInput(sub) => {
                let res = self.parse_expr(pos, rules, sub);
                let new_pos = res.pos;
                res.map(|_| {
                    (HashMap::new(), ActionResult::Value((pos, new_pos)))
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
