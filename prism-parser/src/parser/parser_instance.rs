use crate::core::adaptive::{AdaptError, GrammarState, RuleId};
use crate::core::cache::Allocs;
use crate::core::context::ParserContext;
use crate::core::pos::Pos;
use crate::core::state::ParserState;
use crate::error::aggregate_error::AggregatedParseError;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::grammar::annotated_rule_expr::AnnotatedRuleExpr;
use crate::grammar::charclass::{CharClass, CharClassRange};
use crate::grammar::grammar_file::GrammarFile;
use crate::grammar::rule::Rule;
use crate::grammar::rule_action::RuleAction;
use crate::grammar::rule_annotation::RuleAnnotation;
use crate::grammar::rule_block::RuleBlock;
use crate::grammar::rule_expr::RuleExpr;
use crate::parsable::action_result::ActionResult;
use crate::parsable::parsable_dyn::ParsableDyn;
use crate::parsable::parsed::Parsed;
use crate::parsable::Parsable;
use crate::parser::parsed_list::ParsedList;
use crate::parser::var_map::VarMap;
use crate::META_GRAMMAR;
use std::collections::HashMap;

pub struct ParserInstance<'arn, 'grm: 'arn, E: ParseError<L = ErrorLabel<'grm>>> {
    state: ParserState<'arn, 'grm, E>,

    grammar_state: &'arn GrammarState<'arn, 'grm>,
    rules: VarMap<'arn, 'grm>,
}

impl<'arn, 'grm: 'arn, E: ParseError<L = ErrorLabel<'grm>>> ParserInstance<'arn, 'grm, E> {
    pub fn new(
        input: &'grm str,
        allocs: Allocs<'arn>,
        from: &'arn GrammarFile<'arn, 'grm>,
        mut parsables: HashMap<&'grm str, ParsableDyn<'arn, 'grm>>,
    ) -> Result<Self, AdaptError<'grm>> {
        parsables.insert(
            "ActionResult",
            ParsableDyn::new::<ActionResult<'arn, 'grm>>(),
        );
        parsables.insert("ParsedList", ParsableDyn::new::<ParsedList<'arn, 'grm>>());
        parsables.insert("RuleAction", ParsableDyn::new::<RuleAction<'arn, 'grm>>());
        parsables.insert("CharClass", ParsableDyn::new::<CharClass>());
        parsables.insert("CharClassRange", ParsableDyn::new::<CharClassRange>());
        parsables.insert("RuleAnnotation", ParsableDyn::new::<RuleAnnotation>());
        parsables.insert("RuleExpr", ParsableDyn::new::<RuleExpr>());
        parsables.insert("AnnotatedRuleExpr", ParsableDyn::new::<AnnotatedRuleExpr>());
        parsables.insert("RuleBlock", ParsableDyn::new::<RuleBlock>());
        parsables.insert("Rule", ParsableDyn::new::<Rule>());
        parsables.insert("GrammarFile", ParsableDyn::new::<GrammarFile>());

        let state = ParserState::new(input, allocs, parsables);

        let (grammar_state, meta_vars) = GrammarState::new_with(&META_GRAMMAR, allocs);
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
            allocs,
        );

        let (grammar_state, rules) =
            grammar_state.adapt_with(from, visible_rules, None, state.alloc)?;

        Ok(Self {
            state,
            grammar_state: allocs.alloc(grammar_state),
            rules,
        })
    }
}

impl<'arn, 'grm: 'arn, E: ParseError<L = ErrorLabel<'grm>>> ParserInstance<'arn, 'grm, E> {
    pub fn run(
        &mut self,
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
            self.grammar_state,
            rule,
            &[],
            Pos::start(),
            ParserContext::new(),
        );
        let end_pos = result.end_pos();
        let result = result
            .merge_seq(self.state.parse_end_with_layout(
                self.grammar_state,
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

pub fn run_parser_rule_raw<'arn, 'grm, E: ParseError<L = ErrorLabel<'grm>>>(
    rules: &'arn GrammarFile<'arn, 'grm>,
    rule: &'grm str,
    input: &'grm str,
    allocs: Allocs<'arn>,
    parsables: HashMap<&'grm str, ParsableDyn<'arn, 'grm>>,
) -> Result<Parsed<'arn, 'grm>, AggregatedParseError<'grm, E>> {
    let mut instance: ParserInstance<'arn, 'grm, E> =
        ParserInstance::new(input, allocs, rules, parsables).unwrap();
    instance.run(rule)
}

pub fn run_parser_rule<'arn, 'grm, P: Parsable<'arn, 'grm>, E: ParseError<L = ErrorLabel<'grm>>>(
    rules: &'arn GrammarFile<'arn, 'grm>,
    rule: &'grm str,
    input: &'grm str,
    allocs: Allocs<'arn>,
    parsables: HashMap<&'grm str, ParsableDyn<'arn, 'grm>>,
) -> Result<&'arn P, AggregatedParseError<'grm, E>> {
    run_parser_rule_raw(rules, rule, input, allocs, parsables)
        .map(|parsed| parsed.into_value::<P>())
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
