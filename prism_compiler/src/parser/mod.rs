mod eat;
mod expect;
pub mod lexer;

use crate::lang::diags::ErrorGuaranteed;
use crate::lang::{CoreIndex, CorePrismExpr, PrismDb, ValueOrigin};
use crate::parser::expect::{ErrorState, PResult};
use crate::parser::lexer::{LexerState, Token, Tokens};
use prism_diag_derive::Diagnostic;
use prism_input::input_table::InputTableIndex;
use prism_input::span::Span;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;

impl PrismDb {
    pub fn load_file(&mut self, path: PathBuf) -> Result<InputTableIndex, ErrorGuaranteed> {
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
        self.next_token();
        let program = self.parse_program();
        let tokens = self.finish_lexing();

        let program = match program {
            Ok(program) if matches!(self.token(), Token::EOF(..)) => program,
            _ => {
                let diag = self.expected_into_diag().unwrap();
                let err = self.db.push_error(diag);
                self.db
                    .store(CorePrismExpr::Free, ValueOrigin::Failure(err))
            }
        };

        (program, Arc::new(tokens))
    }

    fn parse_program(&mut self) -> PResult<CoreIndex> {
        self.parse_expr()
    }

    fn parse_expr(&mut self) -> PResult<CoreIndex> {
        self.parse_statement()
    }

    fn parse_statement(&mut self) -> PResult<CoreIndex> {
        if let Ok(kw) = self.eat_keyword("let") {
            let name = self.eat_identifier()?;
            let _ = self.eat_symbol('=')?;
            let value = self.parse_base()?;
            let _ = self.eat_symbol(';')?;
            let body = self.parse_statement()?;
            // let span = kw.start_pos().span_to(self)
            Ok(self.store(CorePrismExpr::Let(), span))
        } else {
            self.parse_base()
        }
    }

    fn parse_base(&mut self) -> PResult<CoreIndex> {
        if let Ok(span) = self.eat_keyword("Type") {
            Ok(self.store(CorePrismExpr::Type, span))
        } else if let Ok(()) = self.eat_paren_open("(") {
            let expr = self.parse_expr()?;
            self.eat_paren_close(")")?;
            Ok(expr)
        } else if let Ok(ident) = self.eat_identifier() {
            Ok(self.store(CorePrismExpr::DeBruijnIndex(0), ident))
        } else {
            Err(self.fail())
        }
    }

    pub fn store(&mut self, e: CorePrismExpr, span: Span) -> CoreIndex {
        self.db.store(e, ValueOrigin::SourceCode(span))
    }
}

#[derive(Diagnostic)]
#[diag(title = format!("Failed to read file `{:?}`: {}", self.path, self.error))]
struct FailedToRead {
    path: PathBuf,
    error: io::Error,
}
