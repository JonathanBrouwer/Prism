use std::collections::HashMap;
use crate::action::{ActionVisitor, ManyVisitor};
use crate::action::action_result::ActionResult;
use crate::core::cache::Allocs;
use crate::grammar::rule_action::RuleAction;
use crate::parser::var_map::VarMap;

pub fn apply_action<'visitor: 'visitor_map, 'visitor_map, 'arn, 'grm>(
    action: &RuleAction<'arn, 'grm>,
    visitor: &'visitor mut dyn ActionVisitor<'arn, 'grm>,
    vars: VarMap<'arn, 'grm>,
    allocs: Allocs<'arn>,
) -> HashMap<&'grm str, &'visitor_map mut dyn ActionVisitor<'arn, 'grm>> {
    let mut map = HashMap::new();
    apply_action_rec(action, visitor, &mut map, vars, allocs);
    map.into_iter().map(|(k, v)| (k, allocs.alloc_unchecked(ManyVisitor(v)) as &mut dyn ActionVisitor)).collect()
}

fn apply_action_rec<'visitor: 'visitor_map, 'visitor_map, 'arn, 'grm>(
    action: &RuleAction<'arn, 'grm>,
    visitor: &'visitor mut dyn ActionVisitor<'arn, 'grm>,
    free_visitors: &mut HashMap<&'grm str, Vec<&'visitor_map mut dyn ActionVisitor<'arn, 'grm>>>,
    vars: VarMap<'arn, 'grm>,
    allocs: Allocs<'arn>,
) {
    match action {
        RuleAction::Name(name) => {
            free_visitors.entry(name).or_default().push(visitor);
        }
        RuleAction::InputLiteral(lit) => {
            visitor.visit_literal(*lit, allocs);
        }
        RuleAction::Construct(name, actions) => {
            let mut visitors = visitor.visit_construct(name, actions.len(), allocs);
            for (sub_visitor, sub_action) in visitors.into_iter().zip(actions.iter()) {
                apply_action_rec(sub_action, sub_visitor, free_visitors, allocs);
            }
        }
        RuleAction::ActionResult(ar) => {
            //TODO horribly unsound
            //TODO add env here
            visitor.visit_cache(*ar as *const ActionResult as *const ())
        }
    }
}
