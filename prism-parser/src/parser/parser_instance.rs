use crate::core::adaptive::{AdaptError, GrammarState};
use crate::core::cache::Allocs;
use crate::core::context::ParserContext;
use crate::core::parser::Parser;
use crate::core::pos::Pos;
use crate::core::state::ParserState;
use crate::error::aggregate_error::AggregatedParseError;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::grammar::action_result::ActionResult;
use crate::grammar::GrammarFile;
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
        let result = parser_rule::parser_rule(&self.grammar_state, rule, &[]).parse(
            Pos::start(),
            &mut self.state,
            self.context,
        );
        let end_pos = result.end_pos();
        let result = result
            .merge_seq(self.state.parse_end_with_layout(
                &self.grammar_state,
                self.rules,
                end_pos,
                self.context,
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
