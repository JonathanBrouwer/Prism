use crate::grammar::RuleAction;
use crate::parser_sugar::action_result::ActionResult;
use std::collections::HashMap;
use std::sync::Arc;

pub fn apply_action<'b, 'grm>(
    rule: &'b RuleAction<'grm>,
    map: &HashMap<&str, Arc<ActionResult<'grm>>>,
) -> Arc<ActionResult<'grm>> {
    match rule {
        RuleAction::Name(name) => {
            if let Some(v) = map.get(&name[..]) {
                v.clone()
            } else {
                panic!("Name '{name}' not in context")
            }
        }
        RuleAction::InputLiteral(lit) => {
            todo!()
            // Arc::new(ActionResult::Literal(lit))
        },
        RuleAction::Construct(name, args) => {
            let args_vals = args.iter().map(|a| apply_action(a, map)).collect();
            Arc::new(ActionResult::Construct(name, args_vals))
        }
        RuleAction::Cons(h, t) => {
            let mut res = Vec::new();
            res.push(apply_action(h, map));
            res.extend_from_slice(match &*apply_action(t, map) {
                ActionResult::List(v) => &v[..],
                x => unreachable!("{:?} is not a list", x),
            });

            Arc::new(ActionResult::List(res))
        }
        RuleAction::Nil() => Arc::new(ActionResult::List(Vec::new())),
    }
}
