use crate::core::adaptive::{AdaptError, GrammarState, RuleId};
use crate::core::cache::Allocs;
use crate::core::context::ParserContext;
use crate::core::cow::Cow;
use crate::core::pos::Pos;
use crate::core::recovery::parse_with_recovery;
use crate::core::state::{PState, ParserState};
use crate::error::aggregate_error::AggregatedParseError;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::grammar::GrammarFile;
use crate::parser::parser_layout::full_input_layout;
use crate::parser::parser_rule;
use crate::rule_action::action_result::ActionResult;
use crate::rule_action::RuleAction;
use crate::META_GRAMMAR_STATE;
use std::collections::HashMap;
pub use typed_arena::Arena;
use crate::parser::var_map::{VarMap, VarMapValue};

pub struct ParserInstance<'arn, 'grm: 'arn, E: ParseError<L = ErrorLabel<'grm>>> {
    context: ParserContext,
    state: PState<'arn, 'grm, E>,

    grammar_state: GrammarState<'arn, 'grm>,
    rules: HashMap<&'grm str, RuleId>,
}

impl<'arn, 'grm: 'arn, E: ParseError<L = ErrorLabel<'grm>>> ParserInstance<'arn, 'grm, E> {
    pub fn new(
        input: &'grm str,
        bump: Allocs<'arn, 'grm>,
        from: &'arn GrammarFile<'grm, RuleAction<'arn, 'grm>>,
    ) -> Result<Self, AdaptError<'grm>> {
        let context = ParserContext::new();
        let cache = ParserState::new(input, bump);

        let visible_rules = [
            ("grammar", META_GRAMMAR_STATE.1["grammar"]),
            ("prule_action", META_GRAMMAR_STATE.1["prule_action"]),
        ]
        .into_iter();

        let (state, rules) = META_GRAMMAR_STATE.0.adapt_with(from, visible_rules, None)?;

        Ok(Self {
            context,
            state: cache,
            grammar_state: state,
            rules: rules.collect(),
        })
    }
}

impl<'arn, 'grm: 'arn, E: ParseError<L = ErrorLabel<'grm>> + 'grm> ParserInstance<'arn, 'grm, E> {
    pub fn run(
        &'arn mut self,
        rule: &'grm str,
    ) -> Result<&'arn ActionResult<'arn, 'grm>, AggregatedParseError<'grm, E>> {
        let rule = self.rules[rule];
        let rule_ctx = VarMap::from_iter(self
            .rules
            .iter()
            .map(|(&k, &v)| (k, VarMapValue::RuleId(v))), &self.state.alloc);
        let result = parse_with_recovery(
            &full_input_layout(
                &self.grammar_state,
                rule_ctx,
                &parser_rule::parser_rule(&self.grammar_state, rule, &[]),
            ),
            Pos::start(),
            &mut self.state,
            &self.context,
        );
        result.map_err(|errors| AggregatedParseError {
            input: self.state.input,
            errors,
        })
    }
}

pub fn run_parser_rule<'arn, 'grm, E: ParseError<L = ErrorLabel<'grm>> + 'grm, T>(
    rules: &'arn GrammarFile<'grm, RuleAction<'arn, 'grm>>,
    rule: &'grm str,
    input: &'grm str,
    ar_map: impl for<'c> FnOnce(&'c ActionResult<'c, 'grm>) -> T,
) -> Result<T, AggregatedParseError<'grm, E>> {
    let allocs: Allocs<'_, 'grm> = Allocs {
        alo_grammarfile: &Arena::new(),
        alo_grammarstate: &Arena::new(),
        alo_ar: &Arena::new(),
        alo_varmap: &Arena::new(),
    };
    let mut instance = ParserInstance::new(input, allocs.clone(), rules).unwrap();
    instance.run(rule).map(ar_map)
}

#[macro_export]
macro_rules! run_parser_rule_here {
    ($id: ident = $rules: expr, $rule: expr, $error: ty, $input: expr) => {
        let bump = $crate::core::cache::Allocs {
            alo_grammarfile: &$crate::parser::parser_instance::Arena::new(),
            alo_grammarstate: &$crate::parser::parser_instance::Arena::new(),
            alo_ar: &$crate::parser::parser_instance::Arena::new(),
            alo_varmap: &$crate::parser::parser_instance::Arena::new(),
        };
        let mut instance =
            $crate::parser::parser_instance::ParserInstance::<$error>::new($input, bump, $rules)
                .unwrap();
        let $id = instance.run($rule);
    };
}
