use crate::core::cow::Cow;
use crate::core::span::Span;
use crate::rule_action::action_result::ActionResult;
use crate::rule_action::RuleAction;

pub fn apply_action<'arn, 'grm>(
    rule: &'arn RuleAction<'arn, 'grm>,
    eval_name: &impl Fn(&str) -> Option<Cow<'arn, ActionResult<'arn, 'grm>>>,
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
            let args_vals = args.iter().map(|a| apply_action(a, eval_name, span)).collect();
            ActionResult::Construct(span, name, args_vals)
        }
        RuleAction::Cons(h, t) => {
            //TODO this is ineffecient
            let mut res = match apply_action(t, eval_name, span).as_ref() {
                ActionResult::Construct(_, "List", v) => v.clone(),
                x => unreachable!("{:?} is not a list", x),
            };
            res.insert(0, apply_action(h, eval_name, span));

            ActionResult::Construct(span, "List", res)
        }
        RuleAction::Nil() => ActionResult::Construct(span, "List", Vec::new()),
        RuleAction::RuleRef(r) => ActionResult::RuleRef(*r),
        RuleAction::ActionResult(ar) => return Cow::Borrowed(ar),
    })
}
