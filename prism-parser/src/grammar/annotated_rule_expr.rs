use crate::core::allocs::alloc_extend;
use crate::core::input::Input;
use crate::core::span::Span;
use crate::grammar::rule_annotation::RuleAnnotation;
use crate::grammar::rule_expr::RuleExpr;
use crate::parsable::Parsable;
use crate::parsable::parsed::Parsed;
use crate::parser::parsed_list::ParsedList;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
pub struct AnnotatedRuleExpr {
    pub annotations: Arc<[Arc<RuleAnnotation>]>,
    pub expr: Arc<RuleExpr>,
}

impl<Db> Parsable<Db> for AnnotatedRuleExpr {
    type EvalCtx = ();

    fn from_construct(_span: Span, constructor: &Input, args: &[Parsed], _env: &mut Db) -> Self {
        assert_eq!("AnnotatedExpr", constructor.as_str());
        Self {
            annotations: alloc_extend(
                args[0]
                    .value_ref::<ParsedList>()
                    .iter()
                    .map(|((), annot)| annot.value_cloned::<RuleAnnotation>()),
            ),
            expr: args[1].value_cloned::<RuleExpr>(),
        }
    }
}
