use crate::core::adaptive::{AdaptResult, GrammarState, RuleId};
use crate::core::cache::{Allocs, PCache, ParserCache};
use crate::core::context::ParserContext;
use crate::core::cow::Cow;
use crate::core::pos::Pos;
use crate::core::recovery::parse_with_recovery;
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

pub struct ParserInstance<'arn, 'grm: 'arn, E: ParseError<L = ErrorLabel<'grm>>> {
    context: ParserContext,
    cache: PCache<'arn, 'grm, E>,

    state: GrammarState<'arn, 'grm>,
    rules: HashMap<&'grm str, RuleId>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Test<'arn> {
    x: Box<Cow<'arn, &'arn Test<'arn>>>,
    s: &'arn str,
}

impl<'arn, 'grm: 'arn, E: ParseError<L = ErrorLabel<'grm>>> ParserInstance<'arn, 'grm, E> {
    pub fn new(
        input: &'grm str,
        bump: Allocs<'arn, 'grm>,
        from: &'arn GrammarFile<'grm, RuleAction<'arn, 'grm>>,
    ) -> Result<Self, AdaptResult<'grm>> {
        let context = ParserContext::new();
        let cache = ParserCache::new(input, bump);

        let visible_rules = [
            ("grammar", META_GRAMMAR_STATE.1["grammar"]),
            ("prule_action", META_GRAMMAR_STATE.1["prule_action"]),
        ]
        .into_iter();

        let (state, rules) = META_GRAMMAR_STATE.0.with(from, visible_rules, None)?;

        Ok(Self {
            context,
            cache,
            state,
            rules: rules.collect(),
        })
    }
}

impl<'arn, 'grm: 'arn, E: ParseError<L = ErrorLabel<'grm>> + 'grm> ParserInstance<'arn, 'grm, E> {
    pub fn run(
        &'arn mut self,
        rule: &'grm str,
    ) -> Result<Cow<'arn, ActionResult<'arn, 'grm>>, Vec<E>> {
        let rule = self.rules[rule];
        let rule_ctx = self
            .rules
            .iter()
            .map(|(&k, &v)| (k, Cow::Owned(ActionResult::RuleRef(v))))
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
        );
        x
    }
}

pub fn run_parser_rule<'arn, 'grm, E: ParseError<L = ErrorLabel<'grm>> + 'grm, T>(
    rules: &'arn GrammarFile<'grm, RuleAction<'arn, 'grm>>,
    rule: &'grm str,
    input: &'grm str,
    ar_map: impl for<'c> FnOnce(&'c ActionResult<'c, 'grm>) -> T,
) -> Result<T, Vec<E>> {
    let allocs: Allocs<'_, 'grm> = Allocs {
        alo_grammarfile: &Arena::new(),
        alo_grammarstate: &Arena::new(),
        alo_ar: &Arena::new(),
    };
    let mut instance = ParserInstance::new(input, allocs.clone(), rules).unwrap();
    instance.run(rule).map(|v| ar_map(allocs.uncow(v)))
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
