use std::collections::HashMap;
use std::{iter, mem};
use crate::core::cache::Allocs;
use crate::core::span::Span;
use crate::action::action_result::ActionResult;
use crate::grammar::rule_action::RuleAction;

pub type FreeMap<'arn, 'grm> = HashMap<&'grm str, Vec<&'arn mut ActionResult<'arn, 'grm>>>;

pub fn apply_action<'arn, 'grm>(
    rule: &RuleAction<'arn, 'grm>,
    free: &mut FreeMap<'arn, 'grm>,
    allocs: Allocs<'arn>,
    current: &'arn mut ActionResult<'arn, 'grm>,
) {
    match rule {
        RuleAction::Name(name) => {
            free.entry(name).or_default().push(current);
        }
        RuleAction::InputLiteral(lit) => {
            *current = ActionResult::Literal(*lit);
        }
        RuleAction::Construct(name, args) => {
            //TODO
            let span = Span::invalid();

            let new_args: &'arn mut [_] = allocs.alloc_extend(iter::repeat_n(*ActionResult::VOID, args.len()));
            let new_args2: &'arn [_] = unsafe { mem::transmute(&*new_args) };
            for (arg, new_arg) in args.iter().zip(new_args.iter_mut()) {
                apply_action(arg, free, allocs, new_arg);
            }
            *current = ActionResult::Construct(span, name, new_args2);
        }
        RuleAction::ActionResult(ar) => {
            //TODO ActionResult::WithEnv(vars, ar)
            todo!()
        }
    }
}
