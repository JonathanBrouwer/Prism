use crate::core::adaptive::ArgsSlice;
use crate::core::allocs::alloc_extend;
use crate::grammar::rule_block::RuleBlock;
use crate::parsable::Parsable;
use crate::parsable::parsed::Parsed;
use crate::parser::parsed_list::ParsedList;
use prism_input::input::Input;
use prism_input::input_table::InputTable;
use prism_input::span::Span;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
pub struct Rule {
    pub name: Input,
    pub adapt: bool,
    pub args: ArgsSlice,
    pub blocks: Arc<[Arc<RuleBlock>]>,
}

impl<Db> Parsable<Db> for Rule {
    type EvalCtx = ();

    fn from_construct(
        _span: Span,
        constructor: &str,
        args: &[Parsed],
        _env: &mut Db,
        _input: &InputTable,
    ) -> Self {
        assert_eq!(constructor, "Rule");

        let parsed = &args[0];
        Rule {
            name: parsed.value_ref::<Input>().clone(),
            adapt: args[1].value_ref::<ParsedList>().iter().next().is_some(),
            args: alloc_extend(
                args[2]
                    .value_ref::<ParsedList>()
                    .iter()
                    .map(|((), v)| v)
                    .map(|n| n.value_ref::<Input>().clone()),
            ),
            blocks: alloc_extend(
                args[3]
                    .value_ref::<ParsedList>()
                    .iter()
                    .map(|((), v)| v)
                    .map(|block| block.value_cloned::<RuleBlock>()),
            ),
        }
    }
}
