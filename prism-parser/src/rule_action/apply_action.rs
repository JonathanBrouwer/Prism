use crate::core::cache::Allocs;
use crate::core::span::Span;
use crate::core::state::PState;
use crate::parser::var_map::{VarMap, VarMapValue};
use crate::rule_action::action_result::ActionResult;
use crate::rule_action::RuleAction;

pub fn apply_action<'arn, 'grm>(
    rule: &RuleAction<'arn, 'grm>,
    span: Span,
    vars: VarMap<'arn, 'grm>,
    allocs: &Allocs<'arn, 'grm>,
) -> &'arn ActionResult<'arn, 'grm> {
    allocs.alo_ar.alloc(match rule {
        RuleAction::Name(name) => {
            if let Some(ar) = vars.get(name) {
                if let VarMapValue::Value(v) = ar {
                    return v.clone();
                } else {
                    panic!("")
                }
            } else {
                panic!("Name '{name}' not in context")
            }
        }
        RuleAction::InputLiteral(lit) => ActionResult::Literal(lit.clone()),
        RuleAction::Construct(name, args) => {
            let args_vals = args.iter().map(|a| apply_action(a, span, vars, allocs)).collect();
            ActionResult::Construct(span, name, args_vals)
        }
        RuleAction::Cons(h, t) => {
            let ar = apply_action(t, span, vars, allocs);
            let ActionResult::Construct(_, "List", ar) = ar else {
                unreachable!("Action result is not a list")
            };
            //TODO this is inefficient
            let mut res = ar.clone();
            res.insert(0, apply_action(h, span, vars, allocs));

            ActionResult::Construct(span, "List", res)
        }
        RuleAction::Nil() => ActionResult::Construct(span, "List", Vec::new()),
        RuleAction::ActionResult(ar) => ActionResult::WithEnv(vars, ar),
    })
}
