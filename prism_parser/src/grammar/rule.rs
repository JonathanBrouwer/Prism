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
    pub args: Arc<[(Input, Input)]>,
    pub blocks: Arc<[Arc<RuleBlock>]>,
    pub return_type: Input,
}

impl<Db> Parsable<Db> for Rule {
    type EvalCtx = ();

    fn from_construct(
        _span: Span,
        constructor: &Input,
        args: &[Parsed],
        _env: &mut Db,
        input: &InputTable,
    ) -> Self {
        assert_eq!(constructor.as_str(input), "Rule");

        let parsed = &args[0];
        Rule {
            name: parsed.value_ref::<Input>().clone(),
            adapt: args[1].value_ref::<ParsedList>().iter().next().is_some(),
            args: alloc_extend(
                args[2]
                    .value_ref::<ParsedList>()
                    .iter()
                    .map(|((), v)| v)
                    .map(|n| {
                        (
                            Input::from_const("ActionResult"),
                            n.value_ref::<Input>().clone(),
                        )
                    }),
            ),
            blocks: alloc_extend(
                args[3]
                    .value_ref::<ParsedList>()
                    .iter()
                    .map(|((), v)| v)
                    .map(|block| block.value_cloned::<RuleBlock>()),
            ),
            return_type: Input::from_const("ActionResult"),
        }
    }

    fn error_fallback(_env: &mut Db, _span: Span) -> Self {
        Self {
            name: Input::from_const(""),
            adapt: false,
            args: Arc::new([]),
            blocks: Arc::new([]),
            return_type: Input::from_const(""),
        }
    }
}
