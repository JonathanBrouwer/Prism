use crate::core::allocs::Allocs;
use crate::grammar::annotated_rule_expr::AnnotatedRuleExpr;
use crate::grammar::grammar_file::GrammarFile;
use crate::grammar::rule::Rule;
use crate::grammar::rule_action::RuleAction;
use crate::grammar::rule_block::RuleBlock;
use crate::grammar::rule_expr::RuleExpr;
use crate::parsable::parsed::Parsed;

impl<'arn, 'grm: 'arn> GrammarFile<'arn, 'grm> {
    pub fn map_actions(
        &'arn self,
        map: &impl Fn(Parsed<'arn, 'grm>) -> Parsed<'arn, 'grm>,
        allocs: Allocs<'arn>,
    ) -> Self {
        Self {
            rules: allocs.alloc_extend(self.rules.iter().map(|r| r.map_actions(map, allocs))),
        }
    }
}

impl<'arn, 'grm: 'arn> Rule<'arn, 'grm> {
    pub fn map_actions(
        self,
        map: &impl Fn(Parsed<'arn, 'grm>) -> Parsed<'arn, 'grm>,
        allocs: Allocs<'arn>,
    ) -> Self {
        Self {
            blocks: allocs.alloc_extend(self.blocks.iter().map(|r| r.map_actions(map, allocs))),
            ..self
        }
    }
}

impl<'arn, 'grm: 'arn> RuleBlock<'arn, 'grm> {
    pub fn map_actions(
        self,
        map: &impl Fn(Parsed<'arn, 'grm>) -> Parsed<'arn, 'grm>,
        allocs: Allocs<'arn>,
    ) -> Self {
        Self {
            constructors: allocs
                .alloc_extend(self.constructors.iter().map(|r| r.map_actions(map, allocs))),
            ..self
        }
    }
}

impl<'arn, 'grm: 'arn> AnnotatedRuleExpr<'arn, 'grm> {
    pub fn map_actions(
        self,
        map: &impl Fn(Parsed<'arn, 'grm>) -> Parsed<'arn, 'grm>,
        allocs: Allocs<'arn>,
    ) -> Self {
        Self {
            expr: self.expr.map_actions(map, allocs),
            ..self
        }
    }
}

impl<'arn, 'grm: 'arn> RuleExpr<'arn, 'grm> {
    pub fn map_actions(
        &'arn self,
        map: &impl Fn(Parsed<'arn, 'grm>) -> Parsed<'arn, 'grm>,
        allocs: Allocs<'arn>,
    ) -> &'arn Self {
        allocs.alloc(match self {
            RuleExpr::Action(e, action) => RuleExpr::Action(
                e.map_actions(map, allocs),
                allocs.alloc(action.map_actions(map, allocs)),
            ),
            RuleExpr::RunVar { rule, args } => RuleExpr::RunVar {
                rule,
                args: allocs.alloc_extend(args.iter().map(|r| *r.map_actions(map, allocs))),
            },
            RuleExpr::Repeat {
                expr,
                min,
                max,
                delim,
            } => RuleExpr::Repeat {
                expr: expr.map_actions(map, allocs),
                min: *min,
                max: *max,
                delim: delim.map_actions(map, allocs),
            },
            RuleExpr::Sequence(es) => RuleExpr::Sequence(
                allocs.alloc_extend(es.iter().map(|r| *r.map_actions(map, allocs))),
            ),
            RuleExpr::Choice(es) => RuleExpr::Choice(
                allocs.alloc_extend(es.iter().map(|r| *r.map_actions(map, allocs))),
            ),
            RuleExpr::NameBind(name, expr) => {
                RuleExpr::NameBind(name, expr.map_actions(map, allocs))
            }
            RuleExpr::SliceInput(expr) => RuleExpr::SliceInput(expr.map_actions(map, allocs)),
            RuleExpr::PosLookahead(expr) => RuleExpr::PosLookahead(expr.map_actions(map, allocs)),
            RuleExpr::NegLookahead(expr) => RuleExpr::NegLookahead(expr.map_actions(map, allocs)),
            RuleExpr::AtAdapt { ns, name, expr } => RuleExpr::AtAdapt {
                ns,
                name,
                expr: expr.map_actions(map, allocs),
            },
            RuleExpr::CharClass(_) | RuleExpr::Literal(_) | RuleExpr::Guid => return self,
        })
    }
}

impl<'arn, 'grm: 'arn> RuleAction<'arn, 'grm> {
    pub fn map_actions(
        self,
        map: &impl Fn(Parsed<'arn, 'grm>) -> Parsed<'arn, 'grm>,
        allocs: Allocs<'arn>,
    ) -> Self {
        match self {
            RuleAction::Name(n) => RuleAction::Name(n),
            RuleAction::InputLiteral(input) => RuleAction::InputLiteral(input),
            RuleAction::Construct { ns, name, args } => RuleAction::Construct {
                ns,
                name,
                args: allocs.alloc_extend(args.iter().map(|r| r.map_actions(map, allocs))),
            },
            RuleAction::Value(v) => RuleAction::Value(map(v)),
        }
    }
}
