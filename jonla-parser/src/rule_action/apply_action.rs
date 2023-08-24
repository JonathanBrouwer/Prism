use crate::core::span::Span;
use crate::rule_action::{ActionResult, RuleAction};
use std::sync::Arc;

pub fn apply_action<'b, 'grm>(
    rule: &'b RuleAction<'grm>,
    map: &impl Fn(&str) -> Option<Arc<ActionResult<'grm>>>,
    span: Span,
) -> Arc<ActionResult<'grm>> {
    match rule {
        RuleAction::Name(name) => {
            if let Some(v) = map(&name[..]) {
                v
            } else {
                panic!("Name '{name}' not in context")
            }
        }
        RuleAction::InputLiteral(lit) => Arc::new(ActionResult::Literal(lit.clone())),
        RuleAction::Construct(name, args) => {
            let args_vals = args.iter().map(|a| apply_action(a, map, span)).collect();
            Arc::new(ActionResult::Construct(span, name, args_vals))
        }
        RuleAction::Cons(h, t) => {
            let mut res = Vec::new();
            res.push(apply_action(h, map, span));
            res.extend_from_slice(match &*apply_action(t, map, span) {
                ActionResult::Construct(_, "List", v) => &v[..],
                x => unreachable!("{:?} is not a list", x),
            });

            Arc::new(ActionResult::Construct(span, "List", res))
        }
        RuleAction::Nil() => Arc::new(ActionResult::Construct(span, "List", Vec::new())),
    }
}
