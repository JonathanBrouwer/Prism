use crate::core::adaptive::BlockState;
use crate::grammar::rule_expr::RuleExpr;
use crate::parsable::Parsable;
use crate::parser::var_map::VarMap;

#[derive(Copy, Clone)]
pub struct RuleClosure<'arn, 'grm> {
    pub expr: &'arn RuleExpr<'arn, 'grm>,
    pub blocks: &'arn [BlockState<'arn, 'grm>],
    pub rule_args: VarMap<'arn, 'grm>,
    pub vars: VarMap<'arn, 'grm>,
}

impl<'arn, 'grm: 'arn> Parsable<'arn, 'grm> for RuleClosure<'arn, 'grm> {}
