use crate::core::allocs::alloc_extend;
use crate::core::input::Input;
use crate::core::span::Span;
use crate::grammar::annotated_rule_expr::AnnotatedRuleExpr;
use crate::parsable::Parsable;
use crate::parsable::parsed::Parsed;
use crate::parser::parsed_list::ParsedList;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
pub struct RuleBlock {
    pub name: Input,
    pub adapt: bool,
    pub constructors: Arc<[Arc<AnnotatedRuleExpr>]>,
}

impl<Db> Parsable<Db> for RuleBlock {
    type EvalCtx = ();

    fn from_construct(_span: Span, constructor: &Input, args: &[Parsed], _env: &mut Db) -> Self {
        assert_eq!(constructor.as_str(), "Block");
        RuleBlock {
            name: Input::from_parsed(&args[0]),
            adapt: args[1].value_ref::<ParsedList>().iter().next().is_some(),
            constructors: alloc_extend(
                args[2]
                    .value_ref::<ParsedList>()
                    .iter()
                    .map(|((), v)| v)
                    .map(|c| c.value_cloned::<AnnotatedRuleExpr>()),
            ),
        }
    }

    fn error_fallback(env: &mut Db, span: Span) -> Self {
        Self {
            name: Input::from_const(""),
            adapt: false,
            constructors: Arc::new([]),
        }
    }
}
