mod eat;
mod expect;
pub mod lexer;

use crate::lang::diags::ErrorGuaranteed;
use crate::lang::{CoreIndex, CorePrismExpr, PrismDb, ValueOrigin};
use crate::parser::expect::ErrorState;
use crate::parser::lexer::{LexerState, Tokens};
use prism_diag_derive::Diagnostic;
use prism_input::input_table::InputTableIndex;
use prism_input::span::Span;
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
    lexer: LexerState,
    error_state: ErrorState,
}

impl<'a> ParserPrismEnv<'a> {
    pub fn new(db: &'a mut PrismDb, file: InputTableIndex) -> Self {
        let pos = db.input.inner().start_of(file);
        Self {
            db,
            lexer: LexerState::new(pos),
            error_state: ErrorState::new(pos),
        }
    }

    pub fn parse_file(mut self) -> (CoreIndex, Arc<Tokens>) {
        while let Some(c) = self.next_real_token() {}
        let tokens = self.finish_lexing();

        let p = self
            .db
            .store(CorePrismExpr::Free, ValueOrigin::SourceCode(Span::dummy()));
        (p, Arc::new(tokens))

        // let start = self.pos;
        //
        // let program = match self.parse_program() {
        //     Ok(program) if self.pos.next(&self.db.input).is_none() => {
        //         self.eat_layout();
        //         program
        //     }
        //     _ => {
        //         let diag = self.expected_into_diag().unwrap();
        //         let err = self.db.push_error(diag);
        //         self.db
        //             .store(CorePrismExpr::Free, ValueOrigin::Failure(err))
        //     }
        // };
        //
        // (program, Arc::new(Tokens(self.tokens)))
    }

    // fn parse_program(&mut self) -> PResult<CoreIndex> {
    //     self.parse_expr()
    // }
    //
    // fn parse_expr(&mut self) -> PResult<CoreIndex> {
    //     self.parse_base()
    // }

    // fn parse_statement(&mut self) -> PResult<()> {
    //     if self.eat_lit("let").is_ok() {
    //         return Ok(())
    //     } else if self.eat_lit("adapt").is_ok() {
    //         return Ok(())
    //     } else {
    //         return Err(self.fail())
    //     }
    // }

    // fn parse_base(&mut self) -> PResult<CoreIndex> {
    //     if let Ok(span) = self.eat_lit("Type") {
    //         Ok(self.store(CorePrismExpr::Type, span))
    //     } else if let Ok(paren_ctx) = self.eat_open_paren("(", ")") {
    //         let expr = self.parse_expr()?;
    //         self.eat_close_paren(paren_ctx)?;
    //         Ok(expr)
    //     } else {
    //         Err(self.fail())
    //     }
    // }

    pub fn store(&mut self, e: CorePrismExpr, span: Span) -> CoreIndex {
        self.db.store(e, ValueOrigin::SourceCode(span))
    }
}
