use std::collections::HashMap;
use std::sync::Arc;
use crate::core::adaptive::{GrammarState, RuleId};
use crate::core::cache::{Allocs, ParserCache, PCache};
use crate::core::context::{ParserContext, Raw, RawEnv};
use crate::core::pos::Pos;
use crate::core::recovery::parse_with_recovery;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::grammar::grammar::{GrammarFile};
use crate::grammar::parser_layout::full_input_layout;
use crate::grammar::parser_rule;
use crate::rule_action::action_result::ActionResult;
use crate::rule_action::apply_action::{apply_rawenv};

pub struct ParserInstance<'b, 'grm: 'b, E: ParseError<L = ErrorLabel<'grm>>> {
    context: ParserContext,
    cache: PCache<'b, 'grm, E>,

    state: GrammarState<'b, 'grm>,
    rules: HashMap<&'grm str, RuleId>,
}

impl<'b, 'grm: 'b, E: ParseError<L = ErrorLabel<'grm>>> ParserInstance<'b, 'grm, E> {
    pub fn new(input: &'grm str, bump: &'b Allocs, from: &'grm GrammarFile<'grm>) -> Self {
        let context = ParserContext::new();
        let cache = ParserCache::new(input, &bump);

        let (state, rules) = GrammarState::new_with(from);

        Self {
            context,
            cache,
            state,
            rules: rules.collect()
        }
    }
}

impl<'b, 'grm: 'b, E: ParseError<L = ErrorLabel<'grm>> + 'grm> ParserInstance<'b, 'grm, E> {
    pub fn run(&'b mut self, rule: &'grm str) -> Result<ActionResult<'grm>, Vec<E>> {
        let rule = self.rules[rule];
        let rule_ctx = self.rules.iter().map(|(&k, &v)| (k, Arc::new(RawEnv::from_raw(Raw::Rule(v))))).collect();
        let x = parse_with_recovery(
            &full_input_layout(
                &self.state,
                &rule_ctx,
                &parser_rule::parser_rule(&self.state, rule, &vec![]),
            ),
            Pos::start(),
            &mut self.cache,
            &self.context,
        ).map(|pr| apply_rawenv(&pr.rtrn));
        x
    }
}


pub fn run_parser_rule<'b, 'grm: 'b, E: ParseError<L = ErrorLabel<'grm>> + 'grm>(
    rules: &'grm GrammarFile<'grm>,
    rule: &'grm str,
    input: &'grm str,
) -> Result<ActionResult<'grm>, Vec<E>> {
    let bump = Allocs::new();
    let mut instance = ParserInstance::new(input, &bump, rules);
    instance.run(rule)
}