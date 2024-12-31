use crate::core::adaptive::BlockState;
use crate::grammar::rule_expr::RuleExpr;
use crate::parsable::ParseResult;
use crate::parser::var_map::VarMap;

#[derive(Copy, Clone)]
pub struct RuleClosure<'arn, 'grm> {
    pub expr: &'arn RuleExpr<'arn, 'grm>,
    pub blocks: &'arn [BlockState<'arn, 'grm>],
    pub rule_args: VarMap<'arn, 'grm>,
    pub vars: VarMap<'arn, 'grm>,
}

impl<'arn, 'grm: 'arn> ParseResult<'arn, 'grm> for RuleClosure<'arn, 'grm> {}
