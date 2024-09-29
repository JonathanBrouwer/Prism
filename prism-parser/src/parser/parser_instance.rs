use crate::core::adaptive::{AdaptError, GrammarState};
use crate::core::cache::Allocs;
use crate::core::context::ParserContext;
use crate::core::pos::Pos;
use crate::core::recovery::parse_with_recovery;
use crate::core::state::ParserState;
use crate::error::aggregate_error::AggregatedParseError;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::grammar::action_result::ActionResult;
use crate::grammar::GrammarFile;
use crate::parser::parser_layout::full_input_layout;
use crate::parser::parser_rule;
use crate::parser::var_map::VarMap;
use crate::META_GRAMMAR;
use bumpalo::Bump;

pub struct ParserInstance<'arn, 'grm: 'arn, E: ParseError<L = ErrorLabel<'grm>>> {
    context: ParserContext,
    state: ParserState<'arn, 'grm, E>,

    grammar_state: GrammarState<'arn, 'grm>,
    rules: VarMap<'arn, 'grm>,
}

impl<'arn, 'grm: 'arn, E: ParseError<L = ErrorLabel<'grm>>> ParserInstance<'arn, 'grm, E> {
    pub fn new(
        input: &'grm str,
        bump: Allocs<'arn>,
        from: &'arn GrammarFile<'arn, 'grm>,
    ) -> Result<Self, AdaptError<'grm>> {
        let context = ParserContext::new();
        let state = ParserState::new(input, bump);

        let (grammar_state, meta_vars) = GrammarState::new_with(&META_GRAMMAR, bump);
        let visible_rules = VarMap::from_iter(
            [
                (
                    "grammar",
                    *meta_vars
                        .get("grammar")
                        .expect("Meta grammar contains 'grammar' rule"),
                ),
                (
                    "prule_action",
                    *meta_vars
                        .get("prule_action")
                        .expect("Meta grammar contains 'prule_action' rule"),
                ),
            ],
            bump,
        );

        let (grammar_state, rules) =
            grammar_state.adapt_with(from, visible_rules, None, state.alloc)?;

        Ok(Self {
            context,
            state,
            grammar_state,
            rules,
        })
    }
}

impl<'arn, 'grm: 'arn, E: ParseError<L = ErrorLabel<'grm>>> ParserInstance<'arn, 'grm, E> {
    pub fn run(
        &'arn mut self,
        rule: &'grm str,
    ) -> Result<&'arn ActionResult<'arn, 'grm>, AggregatedParseError<'grm, E>> {
        let rule = self
            .rules
            .get(rule)
            .expect("Rule exists")
            .as_rule_id()
            .expect("Rule is a rule");
        let result = parse_with_recovery(
            &full_input_layout(
                &self.grammar_state,
                self.rules,
                &parser_rule::parser_rule(&self.grammar_state, rule, &[]),
            ),
            Pos::start(),
            &mut self.state,
            self.context,
        );
        result.map_err(|errors| AggregatedParseError {
            input: self.state.input,
            errors,
        })
    }
}

pub fn run_parser_rule<'arn, 'grm, E: ParseError<L = ErrorLabel<'grm>>, T>(
    rules: &'arn GrammarFile<'arn, 'grm>,
    rule: &'grm str,
    input: &'grm str,
    ar_map: impl for<'c> FnOnce(&'c ActionResult<'c, 'grm>, Allocs<'c>) -> T,
) -> Result<T, AggregatedParseError<'grm, E>> {
    let bump = Bump::new();
    let allocs: Allocs<'_> = Allocs::new(&bump);
    let mut instance = ParserInstance::new(input, allocs, rules).unwrap();
    instance.run(rule).map(|ar| ar_map(ar, allocs))
}

#[macro_export]
macro_rules! run_parser_rule_here {
    ($id: ident = $rules: expr, $rule: expr, $error: ty, $input: expr) => {
        let bump = ::bumpalo::Bump::new();
        let alloc = $crate::core::cache::Allocs::new(&bump);
        let mut instance =
            $crate::parser::parser_instance::ParserInstance::<$error>::new($input, alloc, $rules)
                .unwrap();
        let $id = instance.run($rule);
    };
}
