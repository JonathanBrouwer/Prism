use crate::rule_action::RuleAction;

pub type GrammarFile<'grm> = crate::grammar::GrammarFile<'grm, 'grm, RuleAction<'grm>>;
pub type Rule<'grm> = crate::grammar::Rule<'grm, 'grm, RuleAction<'grm>>;
pub type Block<'grm> = crate::grammar::Block<'grm, 'grm, RuleAction<'grm>>;
pub type AnnotatedRuleExpr<'grm> = crate::grammar::AnnotatedRuleExpr<'grm, 'grm, RuleAction<'grm>>;
pub type RuleExpr<'grm> = crate::grammar::RuleExpr<'grm, 'grm, RuleAction<'grm>>;
