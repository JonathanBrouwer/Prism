use crate::grammar::from_action_result::{parse_identifier, parse_string};
use crate::result_match;
use crate::rule_action::action_result::ActionResult;
use crate::rule_action::action_result::ActionResult::*;
use crate::rule_action::RuleAction;

pub fn parse_rule_action<'arn, 'grm>(
    r: &ActionResult<'_, 'grm>,
    src: &'grm str,
) -> Option<RuleAction<'arn, 'grm>> {
    Some(match r {
        Construct(_, "Construct", b) => RuleAction::Construct(
            parse_identifier(&b[0], src).unwrap(),
            result_match! {
                create b[1].iter_list().map(|sub| parse_rule_action(sub, src)).collect::<Option<Vec<_>>>()?
            }?,
        ),
        Construct(_, "InputLiteral", b) => RuleAction::InputLiteral(parse_string(&b[0], src)?),
        Construct(_, "Name", b) => RuleAction::Name(parse_identifier(&b[0], src)?),
        _ => return None,
    })
}
