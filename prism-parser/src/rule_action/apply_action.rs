use crate::core::cow::Cow;
use crate::core::span::Span;
use crate::rule_action::action_result::ActionResult;
use crate::rule_action::RuleAction;

pub fn apply_action<'b, 'grm>(
    rule: &'b RuleAction<'b, 'grm>,
    map: &impl Fn(&str) -> Option<Cow<'b, ActionResult<'b, 'grm>>>,
    span: Span,
) -> Cow<'b, ActionResult<'b, 'grm>> {
    Cow::Owned(match rule {
        RuleAction::Name(name) => {
            if let Some(ar) = map(name) {
                return ar;
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
            //TODO this is ineffecient
            let mut res = match apply_action(t, map, span).as_ref() {
                ActionResult::Construct(_, "List", v) => v.clone(),
                x => unreachable!("{:?} is not a list", x),
            };
            res.insert(0, apply_action(h, map, span));

            ActionResult::Construct(span, "List", res)
        }
        RuleAction::Nil() => ActionResult::Construct(span, "List", Vec::new()),
        RuleAction::RuleRef(r) => ActionResult::RuleRef(*r),
        RuleAction::ActionResult(ar) => return Cow::Borrowed(ar),
    })
}
