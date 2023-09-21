use std::collections::HashMap;
use crate::core::adaptive::{GrammarState, RuleId};
use crate::core::cache::{Allocs, ParserCache, PCache};
use crate::core::context::{ParserContext};
use crate::core::pos::Pos;
use crate::core::recovery::parse_with_recovery;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::grammar::grammar::{Action, GrammarFile};
use crate::grammar::parser_layout::full_input_layout;
use crate::grammar::parser_rule;
use crate::rule_action::action_result::ActionResult;
use crate::rule_action::apply_action::{apply_rawenv};
use crate::rule_action::RuleAction;

pub struct ParserInstance<'b, 'grm: 'b, E: ParseError<L = ErrorLabel<'grm>> + Clone, A: Action<'grm>> {
    context: ParserContext,
    cache: PCache<'b, 'grm, E>,

    state: GrammarState<'b, 'grm, A>,
    rules: HashMap<&'grm str, RuleId>,
}

impl<'b, 'grm: 'b, E: ParseError<L = ErrorLabel<'grm>> + Clone, A: Action<'grm>> ParserInstance<'b, 'grm, E, A> {
    pub fn new(input: &'grm str, bump: &'b Allocs, from: &'grm GrammarFile<'grm, A>) -> Self {
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

impl<'b, 'grm: 'b, E: ParseError<L = ErrorLabel<'grm>> + Clone> ParserInstance<'b, 'grm, E, RuleAction<'grm>> {
    pub fn run_ar(&'b mut self, rule: &'grm str) -> Result<ActionResult<'grm, RuleAction<'grm>>, Vec<E>> {
        let rule = self.rules[rule];
        let x = parse_with_recovery(
            &full_input_layout(
                &self.state,
                &parser_rule::parser_rule(&self.state, rule, &vec![]),
            ),
            Pos::start(),
            &mut self.cache,
            &self.context,
        ).map(|pr| apply_rawenv(&pr.rtrn));
        x
    }
}



// pub fn run_parser_rule<'b, 'grm: 'b, E: ParseError<L = ErrorLabel<'grm>> + Clone + 'b, A: Action<'grm>>(
//     rules: &'grm GrammarFile<'grm, A>,
//     rule: &'grm str,
//     input: &'grm str,
// ) -> Result<RawEnv<'b, 'grm, A>, Vec<E>> {
//     let bump = Allocs::new();
//     let mut instance = ParserInstance::new(input, &bump, rules);
//     instance.run(rule)
// }

pub fn run_parser_rule_ar<'b, 'grm: 'b, E: ParseError<L = ErrorLabel<'grm>> + Clone>(
    rules: &'grm GrammarFile<'grm, RuleAction<'grm>>,
    rule: &'grm str,
    input: &'grm str,
) -> Result<ActionResult<'grm, RuleAction<'grm>>, Vec<E>> {
    let bump = Allocs::new();
    let mut instance = ParserInstance::new(input, &bump, rules);
    instance.run_ar(rule)
}