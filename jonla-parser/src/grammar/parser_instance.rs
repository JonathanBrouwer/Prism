use crate::core::adaptive::GrammarState;
use crate::core::cache::{Allocs, ParserCache};
use crate::core::context::{ParserContext, PR};
use crate::core::pos::Pos;
use crate::core::presult::PResult;
use crate::core::recovery::parse_with_recovery;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::grammar::grammar::GrammarFile;
use crate::grammar::parser_layout::full_input_layout;
use crate::grammar::parser_rule;

pub struct ParserInstance<'b, 'grm: 'b, E: ParseError<L = ErrorLabel<'grm>> + Clone> {
    context: ParserContext<'b, 'grm>,
    cache: ParserCache<'grm, 'b, PResult<PR<'grm>, E>>,

    state: GrammarState<'b, 'grm>,
}

impl<'b, 'grm: 'b, E: ParseError<L = ErrorLabel<'grm>> + Clone> ParserInstance<'b, 'grm, E> {
    pub fn new(input: &'grm str, bump: &'b Allocs<'b, 'grm>) -> Self {
        let context = ParserContext::new();
        let cache = ParserCache::new(input, &bump);

        let state = GrammarState::new();

        Self {
            context,
            cache,
            state,
        }
    }

    pub fn push_rules(&mut self, grammar: &'grm GrammarFile) -> Result<(), &'grm str> {
        self.state.update(grammar)
    }

    pub fn run(&'b mut self, rule: &'grm str) -> Result<PR<'grm>, Vec<E>> {
        let x = parse_with_recovery(
            &full_input_layout(
                &self.state,
                &parser_rule::parser_rule(&self.state, rule, &vec![]),
            ),
            Pos::start(),
            &mut self.cache,
            &self.context,
        );
        x
    }
}

pub fn run_parser_rule<'b, 'grm: 'b, E: ParseError<L = ErrorLabel<'grm>> + Clone>(
    rules: &'grm GrammarFile,
    rule: &'grm str,
    input: &'grm str,
) -> Result<PR<'grm>, Vec<E>> {
    let bump = Allocs::new();
    let mut instance = ParserInstance::new(input, &bump);
    instance.push_rules(rules).unwrap();
    instance.run(rule)
}
