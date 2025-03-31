use crate::core::allocs::alloc_extend;
use crate::core::input_table::InputTable;
use crate::core::span::Span;
use crate::grammar::identifier::{Identifier, parse_identifier};
use crate::grammar::rule_block::RuleBlock;
use crate::parsable::Parsable;
use crate::parsable::parsed::Parsed;
use crate::parser::parsed_list::ParsedList;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
pub struct Rule {
    pub name: Identifier,
    pub adapt: bool,
    pub args: Arc<[(Identifier, Identifier)]>,
    pub blocks: Arc<[Arc<RuleBlock>]>,
    pub return_type: Identifier,
}

impl<Env> Parsable<Env> for Rule {
    type EvalCtx = ();

    fn from_construct(
        _span: Span,
        constructor: Identifier,
        args: &[Parsed],

        src: &InputTable,
        _env: &mut Env,
    ) -> Self {
        assert_eq!(constructor.as_str(src), "Rule");

        Rule {
            name: parse_identifier(&args[0]),
            adapt: args[1].value_ref::<ParsedList>().iter().next().is_some(),
            args: alloc_extend(
                args[2]
                    .value_ref::<ParsedList>()
                    .iter()
                    .map(|((), v)| v)
                    .map(|n| (Identifier::from_const("ActionResult"), parse_identifier(n))),
            ),
            blocks: alloc_extend(
                args[3]
                    .value_ref::<ParsedList>()
                    .iter()
                    .map(|((), v)| v)
                    .map(|block| block.value_cloned::<RuleBlock>()),
            ),
            return_type: Identifier::from_const("ActionResult"),
        }
    }
}
