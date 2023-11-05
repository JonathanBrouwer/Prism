use crate::core::adaptive::{AdaptResult, BlockState, GrammarState, RuleId, RuleState};
use crate::core::cache::{Allocs, PCache, ParserCache};
use crate::core::context::ParserContext;
use crate::core::cow::Cow;
use crate::core::pos::Pos;
use crate::core::recovery::parse_with_recovery;
use crate::core::span::Span;
use crate::core::toposet::TopoSet;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::grammar::escaped_string::EscapedString;
use crate::grammar::{AnnotatedRuleExpr, GrammarFile};
use crate::parser::parser_layout::full_input_layout;
use crate::parser::parser_rule;
use crate::rule_action::action_result::ActionResult;
use crate::rule_action::RuleAction;
use crate::{META_GRAMMAR, META_GRAMMAR_STATE};
use std::collections::HashMap;
pub use typed_arena::Arena;

pub struct ParserInstance<'b, 'grm: 'b, E: ParseError<L = ErrorLabel<'grm>>> {
    context: ParserContext,
    cache: PCache<'b, 'grm, E>,

    state: GrammarState<'b, 'grm>,
    rules: HashMap<&'grm str, RuleId>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Test<'b> {
    x: Box<Cow<'b, &'b Test<'b>>>,
    s: &'b str,
}

impl<'b, 'grm: 'b, E: ParseError<L = ErrorLabel<'grm>>> ParserInstance<'b, 'grm, E> {
    pub fn new(
        input: &'grm str,
        bump: Allocs<'b, 'grm>,
        from: &'b GrammarFile<'grm, RuleAction<'b, 'grm>>,
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

impl<'b, 'grm: 'b, E: ParseError<L = ErrorLabel<'grm>> + 'grm> ParserInstance<'b, 'grm, E> {
    pub fn run(&'b mut self, rule: &'grm str) -> Result<Cow<'b, ActionResult<'b, 'grm>>, Vec<E>> {
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
        )
        .map(|pr| pr.rtrn);
        x
    }
}

pub fn run_parser_rule<'b, 'grm, E: ParseError<L = ErrorLabel<'grm>> + 'grm, T>(
    rules: &'b GrammarFile<'grm, RuleAction<'b, 'grm>>,
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
