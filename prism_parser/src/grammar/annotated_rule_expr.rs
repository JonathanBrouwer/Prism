use crate::core::allocs::alloc_extend;
use crate::grammar::rule_annotation::RuleAnnotation;
use crate::grammar::rule_expr::RuleExpr;
use crate::parsable::Parsable;
use crate::parsable::parsed::Parsed;
use crate::parser::parsed_list::ParsedList;
use prism_input::input_table::InputTable;
use prism_input::span::Span;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
pub struct AnnotatedRuleExpr {
    pub annotations: Arc<[Arc<RuleAnnotation>]>,
    pub expr: Arc<RuleExpr>,
}

impl<Db> Parsable<Db> for AnnotatedRuleExpr {
    type EvalCtx = ();

    fn from_construct(
        _span: Span,
        constructor: &str,
        args: &[Parsed],
        _env: &mut Db,
        _input: &InputTable,
    ) -> Self {
        assert_eq!("AnnotatedExpr", constructor);
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

    fn error_fallback(env: &mut Db, span: Span) -> Self {
        Self {
            annotations: Arc::new([]),
            expr: Arc::new(RuleExpr::error_fallback(env, span)),
        }
    }
}
