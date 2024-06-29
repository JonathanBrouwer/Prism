use crate::core::cow::Cow;
use crate::core::span::Span;
use crate::rule_action::action_result::ActionResult;
use crate::rule_action::RuleAction;

pub fn apply_action<'arn, 'grm>(
    rule: &RuleAction<'arn, 'grm>,
    eval_name: &mut impl FnMut(&str) -> Option<Cow<'arn, ActionResult<'arn, 'grm>>>,
    span: Span,
) -> Cow<'arn, ActionResult<'arn, 'grm>> {
    Cow::Owned(match rule {
        RuleAction::Name(name) => {
            if let Some(ar) = eval_name(name) {
                return ar;
            } else {
                panic!("Name '{name}' not in context")
            }
        }
        RuleAction::InputLiteral(lit) => ActionResult::Literal(lit.clone()),
        RuleAction::Construct(name, args) => {
            let args_vals = args
                .iter()
                .map(|a| apply_action(a, eval_name, span))
                .collect();
            ActionResult::Construct(span, name, args_vals)
        }
        RuleAction::Cons(h, t) => {
            let ar = apply_action(t, eval_name, span);
            let ActionResult::Construct(_, "List", ar) = ar.as_ref() else {
                unreachable!("Action result is not a list")
            };
            //TODO this is inefficient
            let mut res = ar.clone();
            res.insert(0, apply_action(h, eval_name, span));

            ActionResult::Construct(span, "List", res)
        }
        RuleAction::Nil() => ActionResult::Construct(span, "List", Vec::new()),
        RuleAction::ActionResult(ar) => return Cow::Borrowed(ar),
    })
}
