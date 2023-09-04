use std::sync::Arc;
use crate::core::context::{PR, Raw};
use crate::core::span::Span;
use crate::rule_action::action_result::ActionResult;
use crate::rule_action::RuleAction;

pub fn apply<'b, 'grm>(pr: &PR<'b, 'grm>) -> ActionResult<'grm> {
    match &pr.1 {
        Raw::Internal(_) => panic!("Tried to apply internal raw value."),
        Raw::Value(s) => ActionResult::Value(*s),
        Raw::List(s, l) => ActionResult::Construct(*s, "List", l.iter().map(apply).collect()),
        Raw::Rule(r) => ActionResult::RuleRef(r),
        Raw::Action(a) => apply_action(a, &|n| pr.0.get(n).map(|r| apply(r)), Span::invalid()),
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
            let mut res = Vec::new();
            res.push(apply_action(h, map, span));
            res.extend_from_slice(match &apply_action(t, map, span) {
                ActionResult::Construct(_, "List", v) => &v[..],
                x => unreachable!("{:?} is not a list", x),
            });

            ActionResult::Construct(span, "List", res)
        }
        RuleAction::Nil() => ActionResult::Construct(span, "List", Vec::new()),
    }
}
