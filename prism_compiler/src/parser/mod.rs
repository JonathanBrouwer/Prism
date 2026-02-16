mod eat;
mod expect;
pub mod lexer;

use crate::lang::diags::ErrorGuaranteed;
use crate::lang::{CoreIndex, CorePrismExpr, PrismDb, ValueOrigin};
use crate::parser::expect::{ErrorState, PResult};
use crate::parser::lexer::{LexerState, Token, Tokens};
use prism_data_structures::generic_env::GenericEnv;
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

type NamesEnv = GenericEnv<Span, ()>;

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
        let program = self.parse_program(&NamesEnv::default());
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

    fn parse_program(&mut self, env: &NamesEnv) -> PResult<CoreIndex> {
        self.parse_expr(env)
    }

    fn parse_expr(&mut self, env: &NamesEnv) -> PResult<CoreIndex> {
        self.parse_statement(env)
    }

    fn parse_statement(&mut self, env: &NamesEnv) -> PResult<CoreIndex> {
        if let Ok(kw) = self.eat_keyword("let") {
            let name = self.eat_identifier()?;
            let _ = self.eat_symbol('=')?;
            let value = self.parse_base(env)?;
            let _ = self.eat_symbol(';')?;

            let body_env = env.insert(name, ());
            let body = self.parse_statement(&body_env)?;

            let span = kw.span_to(self.span_of(body));
            Ok(self.store(CorePrismExpr::Let(value, body), span))
        } else {
            self.parse_base(env)
        }
    }

    fn parse_base(&mut self, env: &NamesEnv) -> PResult<CoreIndex> {
        if let Ok(span) = self.eat_keyword("Type") {
            Ok(self.store(CorePrismExpr::Type, span))
        } else if let Ok(()) = self.eat_paren_open("(") {
            let expr = self.parse_expr(env)?;
            self.eat_paren_close(")")?;
            Ok(expr)
        } else if let Ok(found_name_span) = self.eat_identifier() {
            let input = self.db.input.inner();
            let found_name = input.slice(found_name_span);

            if let Some(idx) = env
                .iter()
                .position(|(name, _)| input.slice(*name) == found_name)
            {
                drop(input);
                Ok(self.store(CorePrismExpr::DeBruijnIndex(idx), found_name_span))
            } else {
                drop(input);
                let err = self.db.push_error(UnknownName {
                    span: found_name_span,
                });
                Ok(self
                    .db
                    .store(CorePrismExpr::Free, ValueOrigin::Failure(err)))
            }
        } else {
            Err(self.fail())
        }
    }

    fn span_of(&mut self, e: CoreIndex) -> Span {
        let ValueOrigin::SourceCode(span) = self.db.origins[*e] else {
            unreachable!()
        };
        span
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

#[derive(Diagnostic)]
#[diag(title = "Undefined name within this scope.")]
struct UnknownName {
    #[sugg]
    span: Span,
}
