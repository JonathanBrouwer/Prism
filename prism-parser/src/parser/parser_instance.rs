use crate::META_GRAMMAR;
use crate::core::adaptive::{AdaptError, GrammarState, RuleId};

use crate::core::context::{PV, ParserContext};
use crate::core::input::Input;
use crate::core::input_table::{InputTable, InputTableIndex};
use crate::core::pos::Pos;
use crate::core::state::ParserState;
use crate::core::tokens::Tokens;
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
use crate::parsable::Parsable;
use crate::parsable::action_result::ActionResult;
use crate::parsable::parsable_dyn::ParsableDyn;
use crate::parsable::parsed::ArcExt;
use crate::parsable::void::Void;
use crate::parser::VarMap;
use crate::parser::parsed_list::ParsedList;
use std::collections::HashMap;
use std::sync::Arc;

pub struct ParserInstance<Db, E: ParseError<L = ErrorLabel>> {
    state: ParserState<Db, E>,

    grammar_state: Arc<GrammarState>,
    rules: VarMap,
}

impl<Db, E: ParseError<L = ErrorLabel>> ParserInstance<Db, E> {
    pub fn new(
        input: Arc<InputTable>,
        from: &GrammarFile,
        mut parsables: HashMap<&'static str, ParsableDyn<Db>>,
    ) -> Result<Self, AdaptError> {
        parsables.insert("ActionResult", ParsableDyn::new::<ActionResult>());
        parsables.insert("ParsedList", ParsableDyn::new::<ParsedList>());
        parsables.insert("RuleAction", ParsableDyn::new::<RuleAction>());
        parsables.insert("CharClass", ParsableDyn::new::<CharClass>());
        parsables.insert("CharClassRange", ParsableDyn::new::<CharClassRange>());
        parsables.insert("RuleAnnotation", ParsableDyn::new::<RuleAnnotation>());
        parsables.insert("RuleExpr", ParsableDyn::new::<RuleExpr>());
        parsables.insert("AnnotatedRuleExpr", ParsableDyn::new::<AnnotatedRuleExpr>());
        parsables.insert("RuleBlock", ParsableDyn::new::<RuleBlock>());
        parsables.insert("Rule", ParsableDyn::new::<Rule>());
        parsables.insert("GrammarFile", ParsableDyn::new::<GrammarFile>());
        parsables.insert("OptionU64", ParsableDyn::new::<Option<u64>>());

        let state = ParserState::new(input, parsables);

        let (grammar_state, meta_vars) = GrammarState::new_with(&META_GRAMMAR, &state.input);
        let visible_rules = VarMap::from_iter([
            (
                Input::from_const("grammar"),
                meta_vars
                    .get(&Input::from_const("grammar"))
                    .expect("Meta grammar contains 'grammar' rule")
                    .clone(),
            ),
            (
                Input::from_const("prule_action"),
                meta_vars
                    .get(&Input::from_const("prule_action"))
                    .expect("Meta grammar contains 'prule_action' rule")
                    .clone(),
            ),
        ]);

        let (grammar_state, rules) =
            grammar_state.adapt_with(from, &visible_rules, None, &state.input)?;

        Ok(Self {
            state,
            grammar_state: Arc::new(grammar_state),
            rules,
        })
    }
}

impl<Db, E: ParseError<L = ErrorLabel>> ParserInstance<Db, E> {
    pub fn run(
        &mut self,
        rule: &'static str,
        file: InputTableIndex,
        penv: &mut Db,
    ) -> Result<PV, AggregatedParseError<E>> {
        let rule = *self
            .rules
            .get(&Input::from_const(rule))
            .as_ref()
            .expect("Rule exists")
            .value_ref::<RuleId>();
        let result = self.state.parse_rule(
            &self.grammar_state,
            rule,
            &[],
            Pos::start_of(file),
            ParserContext::new(),
            penv,
            &Arc::new(Void).to_parsed(),
        );
        let end_pos = result.end_pos();
        let result = result
            .merge_seq(self.state.parse_end_with_layout(
                &self.grammar_state,
                &self.rules,
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

pub fn run_parser_rule_raw<Db, E: ParseError<L = ErrorLabel>>(
    rules: &GrammarFile,
    rule: &'static str,
    input: Arc<InputTable>,
    file: InputTableIndex,

    parsables: HashMap<&'static str, ParsableDyn<Db>>,
    penv: &mut Db,
) -> Result<PV, AggregatedParseError<E>> {
    let mut instance: ParserInstance<Db, E> = ParserInstance::new(input, rules, parsables).unwrap();
    instance.run(rule, file, penv)
}

pub fn run_parser_rule<Db, P: Parsable<Db>, E: ParseError<L = ErrorLabel>>(
    rules: &GrammarFile,
    rule: &'static str,
    input_table: Arc<InputTable>,
    file: InputTableIndex,

    parsables: HashMap<&'static str, ParsableDyn<Db>>,
    penv: &mut Db,
) -> Result<(Arc<P>, Arc<Tokens>), AggregatedParseError<E>> {
    run_parser_rule_raw(rules, rule, input_table, file, parsables, penv)
        .map(|parsed| (parsed.parsed.into_value::<P>(), parsed.tokens))
}

#[macro_export]
macro_rules! run_parser_rule_here {
    ($id: ident = $rules: expr, $rule: expr, $error: ty, $input: expr) => {
        let bump = ::bumpalo::OwnedAllocs::default();
        let alloc = $crate::core::allocs::Allocs::new(&bump);
        let mut instance =
            $crate::parser::parser_instance::ParserInstance::<$error>::new($input, $rules).unwrap();
        let $id = instance.run($rule);
    };
}
