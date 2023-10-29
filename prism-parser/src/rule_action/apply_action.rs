use crate::core::context::{Val, ValWithEnv};
use crate::core::span::Span;
use crate::rule_action::action_result::ActionResult;
use crate::rule_action::RuleAction;
use std::collections::HashMap;
use std::sync::Arc;

pub fn apply_rawenv<'grm>(pr: &ValWithEnv<'_, 'grm>) -> ActionResult<'grm> {
    apply(&pr.value, &pr.env)
}

pub fn apply<'b, 'grm>(
    val: &Val<'b, 'grm>,
    env: &HashMap<&'grm str, Arc<ValWithEnv<'b, 'grm>>>,
) -> ActionResult<'grm> {
    match &val {
        Val::Void => panic!("Tried to apply void value."),
        Val::Text(s) => ActionResult::Value(*s),
        Val::List(s, l) => {
            ActionResult::Construct(*s, "List", l.iter().map(|r| apply_rawenv(r)).collect())
        }
        Val::Rule(r) => ActionResult::RuleRef(*r),
        Val::Action(a) => {
            apply_action(a, &|n| env.get(n).map(|r| apply_rawenv(r)), Span::invalid())
        }
    }
}

pub fn apply_action<'b, 'grm>(
    rule: &'b RuleAction<'grm>,
    map: &impl Fn(&str) -> Option<ActionResult<'grm>>,
    span: Span,
) -> ActionResult<'grm> {
    match rule {
        RuleAction::Name(name) => {
            if let Some(v) = map(&name[..]) {
                v
            } else {
                panic!("Name '{name}' not in context")
            }
        }
        RuleAction::InputLiteral(lit) => ActionResult::Literal(lit.clone()),
        RuleAction::Construct(name, args) => {
            let args_vals = args.iter().map(|a| apply_action(a, map, span)).collect();
            ActionResult::Construct(span, name, args_vals)
        }
        RuleAction::Cons(h, t) => {
            let mut res = match apply_action(t, map, span) {
                ActionResult::Construct(_, "List", v) => v,
                x => unreachable!("{:?} is not a list", x),
            };
            //TODO this is ineffecient
            res.insert(0, apply_action(h, map, span));

            ActionResult::Construct(span, "List", res)
        }
        RuleAction::Nil() => ActionResult::Construct(span, "List", Vec::new()),
        RuleAction::RuleRef(r) => ActionResult::RuleRef(*r),
        RuleAction::ActionResult(ar) => (**ar).clone(),
    }
}
