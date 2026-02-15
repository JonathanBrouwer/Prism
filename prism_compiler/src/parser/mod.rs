mod eat;
mod expect;
mod layout;

use crate::lang::diags::ErrorGuaranteed;
use crate::lang::{CoreIndex, CorePrismExpr, PrismDb, ValueOrigin};
use crate::parser::expect::{ErrorState, PResult};
use prism_diag_derive::Diagnostic;
use prism_input::input_table::InputTableIndex;
use prism_input::pos::Pos;
use prism_input::span::Span;
use prism_input::tokens::{Token, Tokens};
use std::io;
use std::path::PathBuf;
use std::sync::Arc;

impl PrismDb {
    pub fn load_file(&mut self, path: PathBuf) -> Result<InputTableIndex, ErrorGuaranteed> {
        #[derive(Diagnostic)]
        #[diag(title = format!("Failed to read file `{:?}`: {}", self.path, self.error))]
        struct FailedToRead {
            path: PathBuf,
            error: io::Error,
        }

        match std::fs::read_to_string(&path) {
            Ok(program) => Ok(self.load_input(program, path)),
            Err(error) => Err(self.push_error(FailedToRead { path, error })),
        }
    }

    pub fn load_input(&mut self, data: String, path: PathBuf) -> InputTableIndex {
        self.input.inner_mut().get_or_push_file(data, path)
    }

    pub fn parse_prism_file(&mut self, file: InputTableIndex) -> (CoreIndex, Arc<Tokens>) {
        let parse_env = ParserPrismEnv::new(self, file);
        parse_env.parse_file()
    }
}

struct ParserPrismEnv<'a> {
    db: &'a mut PrismDb,
    pos: Pos,
    tokens: Vec<Token>,
    error_state: ErrorState,
}

impl<'a> ParserPrismEnv<'a> {
    pub fn new(db: &'a mut PrismDb, file: InputTableIndex) -> Self {
        let pos = db.input.inner().start_of(file);
        Self {
            pos,
            tokens: vec![],
            db,
            error_state: ErrorState::new(pos),
        }
    }

    pub fn parse_file(mut self) -> (CoreIndex, Arc<Tokens>) {
        let start = self.pos;

        _ = self.parse_program();
        self.eat_layout();

        let free = self.db.store(
            CorePrismExpr::Type,
            ValueOrigin::SourceCode(start.span_to(start)),
        );

        if let Some(diag) = self.expected_into_diag() {
            self.db.push_error(diag);
        } else {
            assert!(self.pos.next(&self.db.input).is_none());
        }

        (free, Arc::new(Tokens(self.tokens)))
    }

    fn parse_program(&mut self) -> PResult<()> {
        self.parse_expr()?;

        Ok(())
    }

    // fn parse_statement(&mut self) -> PResult<()> {
    //     if self.eat_lit("let").is_ok() {
    //         return Ok(())
    //     } else if self.eat_lit("adapt").is_ok() {
    //         return Ok(())
    //     } else {
    //         return Err(self.fail())
    //     }
    // }

    fn parse_expr(&mut self) -> PResult<CoreIndex> {
        if let Ok(span) = self.eat_lit("Type") {
            Ok(self.store(CorePrismExpr::Type, span))
        } else {
            Err(self.fail())
        }
    }

    pub fn store(&mut self, e: CorePrismExpr, span: Span) -> CoreIndex {
        self.db.store(e, ValueOrigin::SourceCode(span))
    }
}
