use std::collections::HashMap;
use crate::action::ActionVisitor;
use crate::grammar::rule_action::RuleAction;

pub fn apply_action<'visitor: 'visitor_map, 'visitor_map, 'arn, 'grm>(
    action: &RuleAction<'arn, 'grm>,
    visitor: &'visitor mut dyn ActionVisitor<'arn, 'grm>,
    free_visitors: &mut HashMap<&'grm str, &'visitor_map mut dyn ActionVisitor<'arn, 'grm>>
) {
    match action {
        RuleAction::Name(name) => {
            free_visitors.insert(name, visitor);
        }
        RuleAction::InputLiteral(lit) => {
            visitor.visit_literal(*lit);
        }
        RuleAction::Construct(name, actions) => {
            let mut visitors = visitor.visit_construct(name);
            for (sub_visitor, sub_action) in visitors.iter_mut().zip(actions.iter()) {
                apply_action(sub_action, &mut **sub_visitor, free_visitors);
            }
        }
    }
}