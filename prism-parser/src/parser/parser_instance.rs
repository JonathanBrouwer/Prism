use crate::core::adaptive::{AdaptResult, GrammarState, RuleId};
use crate::core::cache::{Allocs, PCache, ParserCache};
use crate::core::context::ParserContext;
use crate::core::pos::Pos;
use crate::core::recovery::parse_with_recovery;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::grammar::grammar_ar::GrammarFile;
use crate::parser::parser_layout::full_input_layout;
use crate::parser::parser_rule;
use crate::rule_action::action_result::ActionResult;
use crate::META_GRAMMAR_STATE;
use std::collections::HashMap;
pub use typed_arena::Arena;

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
        from: &'grm GrammarFile<'grm, 'grm>,
    ) -> Result<Self, AdaptResult<'grm>> {
        let context = ParserContext::new();
        let cache = ParserCache::new(input, bump);

        let visible_rules = HashMap::from([
            (
                "grammar",
                ActionResult::RuleRef(META_GRAMMAR_STATE.1["grammar"]),
            ),
            (
                "prule_action",
                ActionResult::RuleRef(META_GRAMMAR_STATE.1["prule_action"]),
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
    pub fn run(&'b mut self, rule: &'grm str) -> Result<ActionResult<'b, 'grm>, Vec<E>> {
        let rule = self.rules[rule];
        let rule_ctx = self
            .rules
            .iter()
            .map(|(&k, &v)| (k, ActionResult::RuleRef(v)))
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
        .map(|pr| pr.rtrn);
        x
    }
}

pub fn run_parser_rule<'grm, E: ParseError<L = ErrorLabel<'grm>> + 'grm, T>(
    rules: &'grm GrammarFile<'grm, 'grm>,
    rule: &'grm str,
    input: &'grm str,
    ar_map: impl for<'b> FnOnce(ActionResult<'b, 'grm>) -> T,
) -> Result<T, Vec<E>> {
    let bump: Allocs<'_, 'grm> = Allocs {
        alo_grammarfile: &Arena::new(),
        alo_grammarstate: &Arena::new(),
        alo_ar: &Arena::new(),
    };
    let mut instance = ParserInstance::new(input, bump, rules).unwrap();
    instance.run(rule).map(ar_map)
}

#[macro_export]
macro_rules! run_parser_rule_here {
    ($id: ident = $rules: expr, $rule: expr, $input: expr) => {
        let bump = $crate::core::cache::Allocs {
            alo_grammarfile: &$crate::parser::parser_instance::Arena::new(),
            alo_grammarstate: &$crate::parser::parser_instance::Arena::new(),
            alo_ar: &$crate::parser::parser_instance::Arena::new(),
        };
        let mut instance =
            $crate::parser::parser_instance::ParserInstance::new($input, bump, $rules).unwrap();
        let $id = instance.run($rule);
    };
}
