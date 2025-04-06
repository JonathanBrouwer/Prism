use crate::lang::PrismDb;
use crate::lang::error::PrismError;
use prism_parser::META_GRAMMAR;
use prism_parser::core::input_table::InputTableIndex;
use prism_parser::core::tokens::Tokens;
use prism_parser::error::aggregate_error::AggregatedParseError;
use prism_parser::error::set_error::SetError;
use prism_parser::grammar::grammar_file::GrammarFile;
use prism_parser::parser::parser_instance::run_parser_rule;
use std::collections::HashMap;
use std::sync::Arc;

impl PrismDb {
    pub fn parse_grammar_file(
        &mut self,
        file: InputTableIndex,
    ) -> Result<(Arc<GrammarFile>, Arc<Tokens>), Vec<PrismError>> {
        run_parser_rule::<(), GrammarFile, SetError>(
            &META_GRAMMAR,
            "toplevel",
            self.input.clone(),
            file,
            HashMap::new(),
            &mut (),
        )
        .map_err(|e| e.errors.into_iter().map(PrismError::ParseError).collect())
    }
}
