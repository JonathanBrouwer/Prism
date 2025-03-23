use crate::core::adaptive::BlockState;
use crate::grammar::rule_expr::RuleExpr;
use crate::parsable::ParseResult;
use crate::parser::VarMap;

#[derive(Copy, Clone)]
pub struct RuleClosure<'arn> {
    pub expr: &'arn RuleExpr<'arn>,
    pub blocks: &'arn [BlockState<'arn>],
    pub rule_args: VarMap<'arn>,
    pub vars: VarMap<'arn>,
}

impl ParseResult for RuleClosure<'_> {}
