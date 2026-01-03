use crate::core::adaptive::BlockState;
use crate::core::arc_ref::ArcSlice;
use crate::grammar::rule_expr::RuleExpr;
use crate::parser::VarMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct RuleClosure {
    pub expr: Arc<RuleExpr>,
    pub blocks: ArcSlice<Arc<BlockState>>,
    pub rule_args: VarMap,
    pub vars: VarMap,
}
