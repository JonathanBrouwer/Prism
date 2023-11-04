use crate::rule_action::RuleAction;

pub type GrammarFile<'b, 'grm> = crate::grammar::GrammarFile<'grm, RuleAction<'b, 'grm>>;
pub type Rule<'b, 'grm> = crate::grammar::Rule<'grm, RuleAction<'b, 'grm>>;
pub type Block<'b, 'grm> = crate::grammar::Block<'grm, RuleAction<'b, 'grm>>;
pub type AnnotatedRuleExpr<'b, 'grm> =
    crate::grammar::AnnotatedRuleExpr<'grm, RuleAction<'b, 'grm>>;
pub type RuleExpr<'b, 'grm> = crate::grammar::RuleExpr<'grm, RuleAction<'b, 'grm>>;
