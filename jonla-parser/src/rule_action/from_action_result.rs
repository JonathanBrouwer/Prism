use crate::grammar::from_action_result::{parse_identifier, parse_string};
use crate::result_match;
use crate::rule_action::action_result::ActionResult;
use crate::rule_action::action_result::ActionResult::*;
use crate::rule_action::RuleAction;

pub fn parse_rule_action<'grm>(r: &ActionResult<'grm>, src: &'grm str) -> Option<RuleAction<'grm>> {
    Some(match r {
        Construct(_, "Cons", b) => RuleAction::Cons(
            Box::new(parse_rule_action(&b[0], src)?),
            Box::new(parse_rule_action(&b[1], src)?),
        ),
        Construct(_, "Nil", _) => RuleAction::Nil(),
        Construct(_, "Construct", b) => RuleAction::Construct(
            parse_identifier(&b[0], src)?,
            result_match! {
                match &b[1] => Construct(_, "List", subs),
                create subs.iter().map(|sub| parse_rule_action(sub, src)).collect::<Option<Vec<_>>>()?
            }?,
        ),
        Construct(_, "InputLiteral", b) => RuleAction::InputLiteral(parse_string(&b[0], src)?),
        Construct(_, "Name", b) => RuleAction::Name(parse_identifier(&b[0], src)?),
        _ => return None,
    })
}
