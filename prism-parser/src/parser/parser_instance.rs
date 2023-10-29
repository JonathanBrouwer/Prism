use crate::core::adaptive::{AdaptResult, GrammarState, RuleId};
use crate::core::cache::{Allocs, PCache, ParserCache};
use crate::core::context::{ParserContext, Val, ValWithEnv};
use crate::core::pos::Pos;
use crate::core::recovery::parse_with_recovery;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::grammar::grammar_ar::GrammarFile;
use crate::parser::parser_layout::full_input_layout;
use crate::parser::parser_rule;
use crate::rule_action::action_result::ActionResult;
use crate::rule_action::apply_action::apply_rawenv;
use crate::META_GRAMMAR_STATE;
use std::collections::HashMap;
use std::sync::Arc;
use typed_arena::Arena;

pub struct ParserInstance<'b, 'grm: 'b, E: ParseError<L = ErrorLabel<'grm>>> {
    context: ParserContext,
    cache: PCache<'b, 'grm, E>,

    state: GrammarState<'b, 'grm>,
    rules: HashMap<&'grm str, RuleId>,
}

impl<'b, 'grm: 'b, E: ParseError<L = ErrorLabel<'grm>>> ParserInstance<'b, 'grm, E> {
    pub fn new(
        input: &'grm str,
        bump: Allocs<'b, 'grm>,
        from: &'grm GrammarFile<'grm>,
    ) -> Result<Self, AdaptResult<'grm>> {
        let context = ParserContext::new();
        let cache = ParserCache::new(input, bump);

        let visible_rules = HashMap::from([
            (
                "grammar",
                Arc::new(ValWithEnv::from_raw(Val::Rule(
                    META_GRAMMAR_STATE.1["grammar"],
                ))),
            ),
            (
                "prule_action",
                Arc::new(ValWithEnv::from_raw(Val::Rule(
                    META_GRAMMAR_STATE.1["prule_action"],
                ))),
            ),
        ]);
        let (state, rules) = META_GRAMMAR_STATE.0.with(from, &visible_rules, None)?;

        Ok(Self {
            context,
            cache,
            state,
            rules: rules.collect(),
        })
    }
}

impl<'b, 'grm: 'b, E: ParseError<L = ErrorLabel<'grm>> + 'grm> ParserInstance<'b, 'grm, E> {
    pub fn run(&'b mut self, rule: &'grm str) -> Result<ActionResult<'grm>, Vec<E>> {
        let rule = self.rules[rule];
        let rule_ctx = self
            .rules
            .iter()
            .map(|(&k, &v)| (k, Arc::new(ValWithEnv::from_raw(Val::Rule(v)))))
            .collect();
        let x = parse_with_recovery(
            &full_input_layout(
                &self.state,
                &rule_ctx,
                &parser_rule::parser_rule(&self.state, rule, &[]),
            ),
            Pos::start(),
            &mut self.cache,
            &self.context,
        )
        .map(|pr| apply_rawenv(&pr.rtrn));
        x
    }
}

pub fn run_parser_rule<'b, 'grm: 'b, E: ParseError<L = ErrorLabel<'grm>> + 'grm>(
    rules: &'grm GrammarFile<'grm>,
    rule: &'grm str,
    input: &'grm str,
) -> Result<ActionResult<'grm>, Vec<E>> {
    let bump = Allocs {
        alo_grammarfile: &Arena::new(),
        alo_grammarstate: &Arena::new(),
    };
    let mut instance = ParserInstance::new(input, bump, rules).unwrap();
    instance.run(rule)
}
