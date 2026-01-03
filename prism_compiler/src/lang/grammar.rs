use crate::lang::PrismDb;
use prism_diag::Diag;
use prism_input::input_table::InputTableIndex;
use prism_parser::META_GRAMMAR;
use prism_parser::core::tokens::Tokens;
use prism_parser::error::ParseError;
use prism_parser::error::set_error::SetError;
use prism_parser::grammar::grammar_file::GrammarFile;
use prism_parser::parser::instance::run_parser_rule;
use std::collections::HashMap;
use std::sync::Arc;

impl PrismDb {
    pub fn parse_grammar_file(
        &mut self,
        file: InputTableIndex,
    ) -> (Arc<GrammarFile>, Arc<Tokens>, Vec<Diag>) {
        let (gram, tokens, errs) = run_parser_rule::<(), GrammarFile, SetError>(
            &META_GRAMMAR,
            "toplevel",
            self.input.clone(),
            file,
            HashMap::new(),
            &mut (),
        );
        (
            gram,
            tokens,
            errs.errors.iter().map(SetError::diag).collect(),
        )
    }
}
