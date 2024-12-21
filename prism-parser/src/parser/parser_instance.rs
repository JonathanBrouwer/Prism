use crate::core::adaptive::{AdaptError, GrammarState, RuleId};
use crate::core::cache::Allocs;
use crate::core::context::ParserContext;
use crate::core::pos::Pos;
use crate::core::state::ParserState;
use crate::error::aggregate_error::AggregatedParseError;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::action::action_result::ActionResult;
use crate::grammar::GrammarFile;
use crate::parser::var_map::VarMap;
use crate::META_GRAMMAR;
use bumpalo::Bump;
use crate::core::parsable::{Parsable, Parsed};

pub struct ParserInstance<'arn, 'grm: 'arn, E: ParseError<L = ErrorLabel<'grm>>> {
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
    ) -> Result<Parsed<'arn, 'grm>, AggregatedParseError<'grm, E>> {
        let rule = *self
            .rules
            .get(rule)
            .expect("Rule exists")
            .as_value()
            .expect("Rule is value")
            .into_value::<RuleId>();
        let result = self.state.parse_rule(
            &self.grammar_state,
            rule,
            &[],
            Pos::start(),
            ParserContext::new(),
        );
        let end_pos = result.end_pos();
        let result = result
            .merge_seq(self.state.parse_end_with_layout(
                &self.grammar_state,
                self.rules,
                end_pos,
                ParserContext::new(),
            ))
            .map(|(o, ())| o);

        result.collapse().map_err(|error| AggregatedParseError {
            input: self.state.input,
            errors: vec![error],
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
    instance.run(rule).map(|ar| ar_map(ar.into_value::<ActionResult>(), allocs))
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
