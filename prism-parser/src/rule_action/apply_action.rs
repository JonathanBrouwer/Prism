use crate::core::span::Span;
use crate::rule_action::action_result::ActionResult;
use crate::rule_action::RuleAction;

pub fn apply_action<'b, 'grm>(
    rule: &'b RuleAction<'b, 'grm>,
    map: &impl Fn(&str) -> Option<ActionResult<'b, 'grm>>,
    span: Span,
) -> ActionResult<'b, 'grm> {
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
        RuleAction::ActionResult(ar) => (*ar).clone(),
    }
}
