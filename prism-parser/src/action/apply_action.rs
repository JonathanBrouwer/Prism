use std::collections::HashMap;
use std::{iter, mem};
use crate::core::cache::Allocs;
use crate::core::span::Span;
use crate::action::action_result::ActionResult;
use crate::grammar::rule_action::RuleAction;

pub type FreeMap<'a, 'arn, 'grm> = HashMap<&'grm str, Vec<&'a mut ActionResult<'arn, 'grm>>>;

pub fn apply_action_before<'a, 'b, 'arn: 'a + 'b, 'grm: 'arn + 'a + 'b>(
    action: &RuleAction<'arn, 'grm>,
    free: &'b mut FreeMap<'a, 'arn, 'grm>,
    current: &'a mut ActionResult<'arn, 'grm>,
) {
    match action {
        RuleAction::Name(name) => {
            free.entry(name).or_default().push(current);
        }
        RuleAction::InputLiteral(lit) => {
            *current = ActionResult::Literal(*lit);
        }
        RuleAction::Construct(name, args) => {
            for arg in *args {
                apply_action_before(arg, free);
            }
    
            //TODO
            let span = Span::invalid();
            
            let new_args: &'arn mut [_] = allocs.alloc_extend(iter::repeat_n(*ActionResult::VOID, args.len()));
            let new_args2: &'arn [_] = unsafe { mem::transmute(&*new_args) };
            for (arg, new_arg) in args.iter().zip(new_args.iter_mut()) {
                apply_action_before(arg, free, allocs, new_arg);
            }
            *current = ActionResult::Construct(span, name, new_args2);
        }
        // RuleAction::ActionResult(ar) => {
        //     //TODO ActionResult::WithEnv(vars, ar)
        //     todo!()
        // }
    }
}


// use std::collections::HashMap;
// use std::{iter, mem};
// use crate::core::cache::Allocs;
// use crate::core::span::Span;
// use crate::action::action_result::ActionResult;
// use crate::grammar::rule_action::RuleAction;
//
// pub type FreeMap<'arn, 'grm> = HashMap<&'grm str, Vec<&'arn mut ActionResult<'arn, 'grm>>>;
//
// pub fn apply_action<'arn, 'grm>(
//     rule: &RuleAction<'arn, 'grm>,
//     free: &mut FreeMap<'arn, 'grm>,
//     allocs: Allocs<'arn>,
//     current: &'arn mut ActionResult<'arn, 'grm>,
// ) {
//     match rule {
//         RuleAction::Name(name) => {
//             free.entry(name).or_default().push(current);
//         }
//         RuleAction::InputLiteral(lit) => {
//             *current = ActionResult::Literal(*lit);
//         }
//         RuleAction::Construct(name, args) => {
//             //TODO
//             let span = Span::invalid();
//
//             let new_args: &'arn mut [_] = allocs.alloc_extend(iter::repeat_n(*ActionResult::VOID, args.len()));
//             let new_args2: &'arn [_] = unsafe { mem::transmute(&*new_args) };
//             for (arg, new_arg) in args.iter().zip(new_args.iter_mut()) {
//                 apply_action(arg, free, allocs, new_arg);
//             }
//             *current = ActionResult::Construct(span, name, new_args2);
//         }
//         RuleAction::ActionResult(ar) => {
//             //TODO ActionResult::WithEnv(vars, ar)
//             todo!()
//         }
//     }
// }


// pub fn apply_action_after<'arn, 'grm>(
//     rule: &RuleAction<'arn, 'grm>,
//     span: Span,
//     free: &mut FreeMap<'arn, 'grm>,
//     allocs: Allocs<'arn>,
// ) -> ActionResult<'arn, 'grm> {
//     match rule {
//         RuleAction::Name(name) => {
//             let ar = free[name];
//             assert_ne!(&ar, ActionResult::VOID);
//             ar
//         }
//         RuleAction::InputLiteral(lit) => ActionResult::Literal(*lit),
//         RuleAction::Construct(name, args) => {
//             let args_vals =
//                 allocs.alloc_extend(args.iter().map(|a| apply_action_after(a, span, free, allocs)));
//             ActionResult::Construct(span, name, args_vals)
//         }
//         // RuleAction::ActionResult(ar) => {
//         //     //TODO ActionResult::WithEnv(vars, ar)
//         //     todo!()
//         // }
//     }
// }
