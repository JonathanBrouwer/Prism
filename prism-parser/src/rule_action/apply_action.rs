use crate::core::cache::Allocs;
use crate::core::span::Span;
use crate::parser::var_map::{VarMap, VarMapValue};
use crate::rule_action::action_result::ActionResult;
use crate::rule_action::RuleAction;
use itertools::Itertools;

pub fn apply_action<'arn, 'grm>(
    rule: &RuleAction<'arn, 'grm>,
    span: Span,
    vars: VarMap<'arn, 'grm>,
    allocs: &Allocs<'arn, 'grm>,
) -> ActionResult<'arn, 'grm> {
    match rule {
        RuleAction::Name(name) => {
            if let Some(ar) = vars.get(name) {
                if let VarMapValue::Value(v) = ar {
                    **v
                } else {
                    panic!("")
                }
            } else {
                panic!("Name '{name}' not in context")
            }
        }
        RuleAction::InputLiteral(lit) => ActionResult::Literal(*lit),
        RuleAction::Construct(name, args) => {
            //TODO sucks that we have to make a vec here
            let buffer = args
                .iter()
                .map(|a| apply_action(a, span, vars, allocs))
                .collect_vec();
            let args_vals = allocs.alo_ar.alloc_extend(buffer);
            ActionResult::Construct(span, name, args_vals)
        }
        RuleAction::ActionResult(ar) => ActionResult::WithEnv(vars, ar),
    }
}
