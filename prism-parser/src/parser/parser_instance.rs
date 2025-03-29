use crate::META_GRAMMAR;
use crate::core::adaptive::{AdaptError, GrammarState, RuleId};
use crate::core::allocs::Allocs;
use crate::core::context::ParserContext;
use crate::core::input_table::{InputTable, InputTableIndex};
use crate::core::pos::Pos;
use crate::core::state::ParserState;
use crate::error::ParseError;
use crate::error::aggregate_error::AggregatedParseError;
use crate::error::error_printer::ErrorLabel;
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
use crate::parsable::void::Void;
use crate::parsable::{Parsable, ParseResult};
use crate::parser::VarMap;
use crate::parser::parsed_list::ParsedList;
use std::collections::HashMap;
use std::sync::Arc;

pub struct ParserInstance<'arn, Env, E: ParseError<L = ErrorLabel<'arn>>> {
    state: ParserState<'arn, Env, E>,

    grammar_state: &'arn GrammarState<'arn>,
    rules: VarMap<'arn>,
}

impl<'arn, Env, E: ParseError<L = ErrorLabel<'arn>>> ParserInstance<'arn, Env, E> {
    pub fn new(
        input: Arc<InputTable<'arn>>,
        allocs: Allocs<'arn>,
        from: &'arn GrammarFile<'arn>,
        mut parsables: HashMap<&'arn str, ParsableDyn<'arn, Env>>,
    ) -> Result<Self, AdaptError<'arn>> {
        parsables.insert("ActionResult", ParsableDyn::new::<ActionResult<'arn>>());
        parsables.insert("ParsedList", ParsableDyn::new::<ParsedList<'arn>>());
        parsables.insert("RuleAction", ParsableDyn::new::<RuleAction<'arn>>());
        parsables.insert("CharClass", ParsableDyn::new::<CharClass>());
        parsables.insert("CharClassRange", ParsableDyn::new::<CharClassRange>());
        parsables.insert("RuleAnnotation", ParsableDyn::new::<RuleAnnotation>());
        parsables.insert("RuleExpr", ParsableDyn::new::<RuleExpr>());
        parsables.insert("AnnotatedRuleExpr", ParsableDyn::new::<AnnotatedRuleExpr>());
        parsables.insert("RuleBlock", ParsableDyn::new::<RuleBlock>());
        parsables.insert("Rule", ParsableDyn::new::<Rule>());
        parsables.insert("GrammarFile", ParsableDyn::new::<GrammarFile>());
        parsables.insert("OptionU64", ParsableDyn::new::<Option<u64>>());

        let state = ParserState::new(input, allocs, parsables);

        let (grammar_state, meta_vars) = GrammarState::new_with(&META_GRAMMAR, allocs);
        let visible_rules = VarMap::from_iter(
            [
                (
                    "grammar",
                    meta_vars
                        .get("grammar")
                        .expect("Meta grammar contains 'grammar' rule"),
                ),
                (
                    "prule_action",
                    meta_vars
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

impl<'arn, Env, E: ParseError<L = ErrorLabel<'arn>>> ParserInstance<'arn, Env, E> {
    pub fn run(
        &mut self,
        rule: &'arn str,
        file: InputTableIndex,
        penv: &mut Env,
    ) -> Result<Parsed<'arn>, AggregatedParseError<'arn, E>> {
        let rule = *self
            .rules
            .get(rule)
            .expect("Rule exists")
            .into_value::<RuleId>();
        let result = self.state.parse_rule(
            self.grammar_state,
            rule,
            &[],
            Pos::start_of(file),
            ParserContext::new(),
            penv,
            Void.to_parsed(),
        );
        let end_pos = result.end_pos();
        let result = result
            .merge_seq(self.state.parse_end_with_layout(
                self.grammar_state,
                self.rules,
                end_pos,
                ParserContext::new(),
                penv,
            ))
            .map(|(o, ())| o);

        result.collapse().map_err(|error| AggregatedParseError {
            input: self.state.input.clone(),
            errors: vec![error],
        })
    }
}

pub fn run_parser_rule_raw<'a, 'arn, Env, E: ParseError<L = ErrorLabel<'arn>>>(
    rules: &'arn GrammarFile<'arn>,
    rule: &'arn str,
    input: Arc<InputTable<'arn>>,
    file: InputTableIndex,
    allocs: Allocs<'arn>,
    parsables: HashMap<&'arn str, ParsableDyn<'arn, Env>>,
    penv: &'a mut Env,
) -> Result<Parsed<'arn>, AggregatedParseError<'arn, E>> {
    let mut instance: ParserInstance<'arn, Env, E> =
        ParserInstance::new(input, allocs, rules, parsables).unwrap();
    instance.run(rule, file, penv)
}

pub fn run_parser_rule<
    'a,
    'arn,
    Env,
    P: Parsable<'arn, Env>,
    E: ParseError<L = ErrorLabel<'arn>>,
>(
    rules: &'arn GrammarFile<'arn>,
    rule: &'arn str,
    input: &'arn str,
    allocs: Allocs<'arn>,
    parsables: HashMap<&'arn str, ParsableDyn<'arn, Env>>,
    penv: &'a mut Env,
) -> Result<&'arn P, AggregatedParseError<'arn, E>> {
    let input_table = InputTable::default();
    let file = input_table.get_or_push_file(input, "input".into());

    run_parser_rule_raw(
        rules,
        rule,
        Arc::new(input_table),
        file,
        allocs,
        parsables,
        penv,
    )
    .map(|parsed| parsed.into_value::<P>())
}

#[macro_export]
macro_rules! run_parser_rule_here {
    ($id: ident = $rules: expr, $rule: expr, $error: ty, $input: expr) => {
        let bump = ::bumpalo::OwnedAllocs::default();
        let alloc = $crate::core::allocs::Allocs::new(&bump);
        let mut instance =
            $crate::parser::parser_instance::ParserInstance::<$error>::new($input, alloc, $rules)
                .unwrap();
        let $id = instance.run($rule);
    };
}
