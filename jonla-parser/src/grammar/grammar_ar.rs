use crate::rule_action::RuleAction;

pub type GrammarFile<'grm> = crate::grammar::GrammarFile<'grm, RuleAction<'grm>>;
pub type Rule<'grm> = crate::grammar::Rule<'grm, RuleAction<'grm>>;
pub type Block<'grm> = crate::grammar::Block<'grm, RuleAction<'grm>>;
pub type AnnotatedRuleExpr<'grm> = crate::grammar::AnnotatedRuleExpr<'grm, RuleAction<'grm>>;
pub type RuleExpr<'grm> = crate::grammar::RuleExpr<'grm, RuleAction<'grm>>;
