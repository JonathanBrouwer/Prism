use crate::grammar::{RuleAction, RuleBody};
use crate::parser::parser_core::ParserState;
use crate::parser::parser_result::ParseResult;
use itertools::Itertools;
use std::collections::HashMap;

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
    ) -> ParseResult<(HashMap<&'grm str, ActionResult<'grm>>, ActionResult<'grm>)> {
        self.parse_expr(pos, rules, &rules.get(rule).unwrap())
    }

    pub fn parse_expr(
        &mut self,
        pos: usize,
        rules: &HashMap<&'grm str, RuleBody<'grm>>,
        expr: &RuleBody<'grm>,
    ) -> ParseResult<(HashMap<&'grm str, ActionResult<'grm>>, ActionResult<'grm>)> {
        match expr {
            RuleBody::Rule(_) => {
                todo!()
            }
            RuleBody::CharClass(cc) => self
                .parse_charclass(pos, cc)
                .map(|x| (HashMap::new(), ActionResult::Value(x))),
            RuleBody::Literal(_) => {
                todo!()
            }
            RuleBody::Repeat { .. } => {
                todo!()
            }
            RuleBody::Sequence(_) => {
                todo!()
            }
            RuleBody::Choice(_) => {
                todo!()
            }
            RuleBody::NameBind(name, sub) => {
                let mut res = self.parse_expr(pos, rules, sub);
                res.result.0.insert(name, res.result.1.clone());
                res
            }
            RuleBody::Action(sub, action) => {
                let mut res = self.parse_expr(pos, rules, sub);
                res.result.1 = apply_action(action, &res.result.0);
                res
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
