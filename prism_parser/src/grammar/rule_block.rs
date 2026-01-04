use crate::core::allocs::alloc_extend;
use crate::grammar::annotated_rule_expr::AnnotatedRuleExpr;
use crate::parsable::Parsable;
use crate::parsable::parsed::Parsed;
use crate::parser::parsed_list::ParsedList;
use prism_input::input::Input;
use prism_input::input_table::InputTable;
use prism_input::span::Span;
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

    fn from_construct(
        _span: Span,
        constructor: &str,
        args: &[Parsed],
        _env: &mut Db,
        _input: &InputTable,
    ) -> Self {
        assert_eq!(constructor, "Block");
        let parsed = &args[0];
        RuleBlock {
            name: parsed.value_ref::<Input>().clone(),
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
}
