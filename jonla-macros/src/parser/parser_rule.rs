use crate::grammar::{CharClass, RuleAction, RuleBody};
use crate::parser::parser_core::ParserState;
use crate::parser::parser_result::{ParseErrorLabel, ParseResult};
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

impl<'grm> ActionResult<'grm> {
    pub fn to_string<'src>(&self, src: &'src str) -> String {
        match self {
            ActionResult::Value((s, e)) => format!("\'{}\'", &src[*s..*e]),
            ActionResult::Literal(lit) => format!("\'{}\'", lit.to_string()),
            ActionResult::Construct(c, es) => format!("{}({})", c, es.iter().map(|e| e.to_string(src)).format(", ")),
            ActionResult::Error => format!("ERROR"),
        }
    }
}

impl<'grm, 'src> ParserState<'grm, 'src, PR<'grm>> {
    pub fn parse_rule(
        &mut self,
        pos: usize,
        rules: &HashMap<&'grm str, RuleBody<'grm>>,
        rule: &'grm str,
    ) -> ParseResult<'grm, PR<'grm>> {
        self.parse_cache_recurse(
            pos,
            |s, p| s.parse_expr(p, rules, &rules.get(rule).unwrap()),
            rule,
        )
    }

    pub fn parse_expr(
        &mut self,
        pos: usize,
        rules: &HashMap<&'grm str, RuleBody<'grm>>,
        expr: &RuleBody<'grm>,
    ) -> ParseResult<'grm, PR<'grm>> {
        match expr {
            RuleBody::Rule(rule) => self
                .parse_rule(pos, rules, rule)
                .map(|(_, v)| (HashMap::new(), v)),
            RuleBody::CharClass(cc) => {
                let result = self.parse_charclass(pos, cc);
                result.map_with_pos(|_, new_pos| {
                    (HashMap::new(), ActionResult::Value((pos, new_pos)))
                })
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
                state.map_with_pos(|_, new_pos| {
                    (HashMap::new(), ActionResult::Value((pos, new_pos)))
                }).map_errs(|mut err| {
                    err.labels = vec![ParseErrorLabel::Error(literal)];
                    err.start = Some(pos);
                    err
                })
            }
            RuleBody::Repeat {
                expr,
                min,
                max,
                delim,
            } => {
                let mut state = ParseResult::new_ok((HashMap::new(), ()), pos);
                let mut results = vec![];

                for i in 0..max.unwrap_or(u64::MAX) {
                    let mut state_new = state.clone();

                    //Parse delim
                    if i != 0 {
                        let res = self.parse_sequence(state_new.clone(), |s, p| {
                            s.parse_expr(p, rules, delim)
                        });
                        state_new = res.map(|(l, _)| l)
                    }

                    //Parse expr
                    let res =
                        self.parse_sequence(state_new.clone(), |s, p| s.parse_expr(p, rules, expr));

                    match res.inner {
                        //If we can stop, do so
                        Err(err) if i >= *min => {
                            state.add_error_info(err);
                            break;
                        }
                        _ => {
                            //Update state
                            state = res.map(|(l, r)| {
                                results.push(r.1);
                                l
                            });
                        }
                    }
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
                //TODO should empty choices be allowed? If so, what error should that give?
                let mut state = ParseResult::new_err(pos, vec![]);
                for sub in subs {
                    state = self.parse_choice(pos, state, |s, p| s.parse_expr(p, rules, sub));
                }
                state.map(|(_, v)| (HashMap::new(), v))
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
                let new_pos = res.pos();
                res.map(|_| (HashMap::new(), ActionResult::Value((pos, new_pos))))
            }
            RuleBody::Error(sub, err_label) => {
                let res = self.parse_expr(pos, rules, sub);
                res.map_errs(|mut err| {
                    err.labels = vec![ParseErrorLabel::Error(err_label)];
                    err.start = Some(pos);
                    err
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
