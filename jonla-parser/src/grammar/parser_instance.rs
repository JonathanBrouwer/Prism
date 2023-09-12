use crate::core::adaptive::GrammarState;
use crate::core::cache::{Allocs, ParserCache};
use crate::core::context::{ParserContext, PR, RawEnv};
use crate::core::pos::Pos;
use crate::core::presult::PResult;
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
    context: ParserContext<'b, 'grm, A>,
    cache: ParserCache<'grm, 'b, PResult<PR<'b, 'grm, A>, E>, A>,

    state: GrammarState<'b, 'grm, A>,
}

impl<'b, 'grm: 'b, E: ParseError<L = ErrorLabel<'grm>> + Clone, A: Action<'grm>> ParserInstance<'b, 'grm, E, A> {
    pub fn new(input: &'grm str, bump: &'b Allocs, from: &'grm GrammarFile<'grm, A>) -> Self {
        let context = ParserContext::new();
        let cache = ParserCache::new(input, &bump);

        let state = GrammarState::new_with(from);

        Self {
            context,
            cache,
            state,
        }
    }

    pub fn run(&'b mut self, rule: &'grm str) -> Result<RawEnv<'b, 'grm, A>, Vec<E>> {
        let x = parse_with_recovery(
            &full_input_layout(
                &self.state,
                &parser_rule::parser_rule(&self.state, rule, &vec![]),
            ),
            Pos::start(),
            &mut self.cache,
            &self.context,
        ).map(|pr| pr.rtrn);
        x
    }
}

impl<'b, 'grm: 'b, E: ParseError<L = ErrorLabel<'grm>> + Clone> ParserInstance<'b, 'grm, E, RuleAction<'grm>> {
    pub fn run_ar(&'b mut self, rule: &'grm str) -> Result<ActionResult<'grm>, Vec<E>> {
        let r = self.run(rule)?;
        Ok(apply_rawenv(&r, &self.state))
    }
}



pub fn run_parser_rule<'b, 'grm: 'b, E: ParseError<L = ErrorLabel<'grm>> + Clone, A: Action<'grm>>(
    rules: &'grm GrammarFile<'grm, A>,
    rule: &'grm str,
    input: &'grm str,
) -> Result<RawEnv<'b, 'grm, A>, Vec<E>> {
    let bump = Allocs::new();
    let mut instance = ParserInstance::new(input, &bump, rules);
    instance.run(rule)
}

pub fn run_parser_rule_ar<'b, 'grm: 'b, E: ParseError<L = ErrorLabel<'grm>> + Clone>(
    rules: &'grm GrammarFile<'grm, RuleAction<'grm>>,
    rule: &'grm str,
    input: &'grm str,
) -> Result<ActionResult<'grm>, Vec<E>> {
    let bump = Allocs::new();
    let mut instance = ParserInstance::new(input, &bump, rules);
    instance.run_ar(rule)
}