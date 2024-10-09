use std::collections::HashMap;
use crate::core::cache::Allocs;
use crate::core::span::Span;
use crate::action::action_result::ActionResult;
use crate::grammar::rule_action::RuleAction;
use crate::parser::var_map::{VarMap, VarMapValue};

pub fn apply_action<'arn, 'grm>(
    rule: &RuleAction<'arn, 'grm>,
    span: Span,
    vars: VarMap<'arn, 'grm>,
    allocs: &Allocs<'arn>,
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
            let args_vals =
                allocs.alloc_extend(args.iter().map(|a| apply_action(a, span, vars, allocs)));
            ActionResult::Construct(span, name, args_vals)
        }
    }
}

// fn apply_action_v2<'arn, 'grm>(
//     rule: &RuleAction<'arn, 'grm>,
//     allocs: &Allocs<'arn>,
//     map: &mut HashMap<&'grm str, &'arn ActionResult<'arn, 'grm>>,
//     ar:
// ) -> ActionResult<'arn, 'grm> {
//
//
//
//     match rule {
//         RuleAction::Name(name) => {
//             map.insert(name, )
//
//             if let Some(ar) = vars.get(name) {
//                 if let VarMapValue::Value(v) = ar {
//                     **v
//                 } else {
//                     panic!("")
//                 }
//             } else {
//                 panic!("Name '{name}' not in context")
//             }
//         }
//         RuleAction::InputLiteral(lit) => ActionResult::Literal(*lit),
//         RuleAction::Construct(name, args) => {
//             let args_vals =
//                 allocs.alloc_extend(args.iter().map(|a| crate::action::apply_action::apply_action(a, span, vars, allocs)));
//             ActionResult::Construct(span, name, args_vals)
//         }
//         RuleAction::ActionResult(ar) => ActionResult::WithEnv(vars, ar),
//     }
// }