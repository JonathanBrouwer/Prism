use crate::core::allocs::alloc_extend;
use crate::grammar::annotated_rule_expr::AnnotatedRuleExpr;
use crate::grammar::grammar_file::GrammarFile;
use crate::grammar::rule::Rule;
use crate::grammar::rule_action::RuleAction;
use crate::grammar::rule_block::RuleBlock;
use crate::grammar::rule_expr::RuleExpr;
use crate::parsable::parsed::Parsed;
use std::sync::Arc;

impl GrammarFile {
    pub fn map_actions(&self, map: &impl Fn(&Parsed) -> Parsed) -> Arc<Self> {
        Arc::new(Self {
            rules: alloc_extend(self.rules.iter().map(|r| r.map_actions(map))),
        })
    }
}

impl Rule {
    pub fn map_actions(&self, map: &impl Fn(&Parsed) -> Parsed) -> Arc<Self> {
        Arc::new(Self {
            blocks: alloc_extend(self.blocks.iter().map(|r| r.map_actions(map))),

            name: self.name.clone(),
            adapt: self.adapt,
            args: self.args.clone(),
            return_type: self.return_type.clone(),
        })
    }
}

impl RuleBlock {
    pub fn map_actions(&self, map: &impl Fn(&Parsed) -> Parsed) -> Arc<Self> {
        Arc::new(Self {
            constructors: alloc_extend(self.constructors.iter().map(|r| r.map_actions(map))),

            name: self.name.clone(),
            adapt: self.adapt,
        })
    }
}

impl AnnotatedRuleExpr {
    pub fn map_actions(&self, map: &impl Fn(&Parsed) -> Parsed) -> Arc<Self> {
        Arc::new(Self {
            expr: self.expr.map_actions(map),

            annotations: self.annotations.clone(),
        })
    }
}

impl RuleExpr {
    pub fn map_actions(self: &Arc<Self>, map: &impl Fn(&Parsed) -> Parsed) -> Arc<Self> {
        Arc::new(match &**self {
            RuleExpr::Action(e, action) => {
                RuleExpr::Action(e.map_actions(map), action.map_actions(map))
            }
            RuleExpr::RunVar { rule, args } => RuleExpr::RunVar {
                rule: rule.clone(),
                args: alloc_extend(args.iter().map(|r| r.map_actions(map))),
            },
            RuleExpr::Repeat {
                expr,
                min,
                max,
                delim,
            } => RuleExpr::Repeat {
                expr: expr.map_actions(map),
                min: *min,
                max: *max,
                delim: delim.map_actions(map),
            },
            RuleExpr::Sequence(es) => {
                RuleExpr::Sequence(alloc_extend(es.iter().map(|r| r.map_actions(map))))
            }
            RuleExpr::Choice(es) => {
                RuleExpr::Choice(alloc_extend(es.iter().map(|r| r.map_actions(map))))
            }
            RuleExpr::NameBind(name, expr) => {
                RuleExpr::NameBind(name.clone(), expr.map_actions(map))
            }
            RuleExpr::SliceInput(expr) => RuleExpr::SliceInput(expr.map_actions(map)),
            RuleExpr::PosLookahead(expr) => RuleExpr::PosLookahead(expr.map_actions(map)),
            RuleExpr::NegLookahead(expr) => RuleExpr::NegLookahead(expr.map_actions(map)),
            RuleExpr::AtAdapt { ns, name, expr } => RuleExpr::AtAdapt {
                ns: ns.clone(),
                name: name.clone(),
                expr: expr.map_actions(map),
            },
            RuleExpr::CharClass(_) | RuleExpr::Literal(_) | RuleExpr::Guid => return self.clone(),
        })
    }
}

impl RuleAction {
    pub fn map_actions(self: &Arc<Self>, map: &impl Fn(&Parsed) -> Parsed) -> Arc<Self> {
        Arc::new(match &**self {
            RuleAction::Name(..) | &RuleAction::InputLiteral(..) => return self.clone(),
            RuleAction::Construct { ns, name, args } => RuleAction::Construct {
                ns: ns.clone(),
                name: name.clone(),
                args: alloc_extend(args.iter().map(|r| r.map_actions(map))),
            },
            RuleAction::Value { ns, value } => RuleAction::Value {
                ns: ns.clone(),
                value: map(value),
            },
        })
    }
}
