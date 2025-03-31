use crate::core::allocs::alloc_extend;
use crate::core::input_table::InputTable;
use crate::core::span::Span;
use crate::grammar::annotated_rule_expr::AnnotatedRuleExpr;
use crate::grammar::identifier::{Identifier, parse_identifier};
use crate::parsable::Parsable;
use crate::parsable::parsed::Parsed;
use crate::parser::parsed_list::ParsedList;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
pub struct RuleBlock {
    pub name: Identifier,
    pub adapt: bool,
    pub constructors: Arc<[Arc<AnnotatedRuleExpr>]>,
}

impl<Env> Parsable<Env> for RuleBlock {
    type EvalCtx = ();

    fn from_construct(
        _span: Span,
        constructor: Identifier,
        args: &[Parsed],

        src: &InputTable,
        _env: &mut Env,
    ) -> Self {
        assert_eq!(constructor.as_str(src), "Block");
        RuleBlock {
            name: parse_identifier(&args[0]),
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
