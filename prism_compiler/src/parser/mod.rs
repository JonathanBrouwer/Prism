mod eat;
mod expect;
pub mod lexer;

use crate::lang::diags::ErrorGuaranteed;
use crate::lang::{CoreIndex, CorePrismExpr, PrismDb, ValueOrigin};
use crate::parser::expect::{ErrorState, Expected, PResult};
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
        let program = self.parse_program(&NamesEnv::default());
        let tokens = self.finish_lexing();

        let program = match program {
            Ok(program)
                if self
                    .eat_token(Expected::EOF, |t, _| matches!(t, Token::EOF(..)))
                    .is_ok() =>
            {
                program
            }
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
            let typ = if let Ok(_) = self.eat_symbol(':') {
                let typ = self.parse_fnconstruct(env)?;
                Some(typ)
            } else {
                None
            };

            let _ = self.eat_symbol('=')?;
            let value = self.parse_fnconstruct(env)?;
            let _ = self.eat_symbol(';')?;

            let body_env = env.insert(name, ());
            let body = self.parse_statement(&body_env)?;

            let span = kw.span_to(self.span_of(body));
            let value = if let Some(typ) = typ {
                let assert_span = name.span_to(self.span_of(value));
                self.store(CorePrismExpr::TypeAssert(value, typ), assert_span)
            } else {
                value
            };
            Ok(self.store(CorePrismExpr::Let(value, body), span))
        } else {
            self.parse_fnconstruct(env)
        }
    }

    fn parse_fnconstruct(&mut self, env: &NamesEnv) -> PResult<CoreIndex> {
        if let Ok((start, binding, typ)) = self.try_parse(|parser| {
            let start = parser.eat_paren_open("(")?;
            let binding = parser.eat_identifier()?;
            let typ = if let Ok(_) = parser.eat_symbol(':') {
                Some(parser.parse_fntype(env)?)
            } else {
                None
            };
            parser.eat_paren_close(")")?;
            parser.eat_multi_symbol("=>")?;
            Ok((start, binding, typ))
        }) {
            let body_env = env.insert(binding, ());
            // Insert dummy entry for let binding of assert
            let body_env = if typ.is_some() {
                body_env.insert(Span::new(binding.start_pos(), 0), ())
            } else {
                body_env
            };
            let body = self.parse_fnconstruct(&body_env)?;

            let span = start.span_to(self.span_of(body));
            let body = if let Some(typ) = typ {
                let assert_span = binding.span_to(self.span_of(typ));
                let var_ref = self.store(CorePrismExpr::DeBruijnIndex(0), assert_span);
                let typ_assert = self.store(CorePrismExpr::TypeAssert(var_ref, typ), assert_span);
                self.store(CorePrismExpr::Let(typ_assert, body), assert_span)
            } else {
                body
            };
            Ok(self.store(CorePrismExpr::FnConstruct(body), span))
        } else {
            self.parse_fntype(env)
        }
    }

    fn parse_fntype(&mut self, env: &NamesEnv) -> PResult<CoreIndex> {
        if let Ok((start, binding, typ)) = self.try_parse(|parser| {
            let start = parser.eat_paren_open("(")?;
            let binding = parser.eat_identifier()?;
            _ = parser.eat_symbol(':')?;
            let typ = parser.parse_fndestruct(env)?;
            parser.eat_paren_close(")")?;
            parser.eat_multi_symbol("->")?;
            Ok((start, binding, typ))
        }) {
            let body_env = env.insert(binding, ());
            let body = self.parse_fndestruct(&body_env)?;

            let span = start.span_to(self.span_of(body));
            Ok(self.store(CorePrismExpr::FnType(typ, body), span))
        } else if let Ok(typ) = self.try_parse(|parser| {
            let typ = parser.parse_fndestruct(env)?;
            parser.eat_multi_symbol("->")?;
            Ok(typ)
        }) {
            let body = self.parse_fndestruct(env)?;
            let span = self.span_of(typ).span_to(self.span_of(body));
            Ok(self.store(CorePrismExpr::FnType(typ, body), span))
        } else {
            self.parse_assert(env)
        }
    }

    fn parse_assert(&mut self, env: &NamesEnv) -> PResult<CoreIndex> {
        self.parse_fndestruct(env)
    }

    fn parse_fndestruct(&mut self, env: &NamesEnv) -> PResult<CoreIndex> {
        let mut current = self.parse_base(env)?;

        loop {
            match self.try_parse(|parser| parser.parse_base(env)) {
                Ok(next) => {
                    let span = self.span_of(current).span_to(self.span_of(next));
                    current = self.store(CorePrismExpr::FnDestruct(current, next), span);
                }
                Err(_err) => break,
            }
        }

        Ok(current)
    }

    fn parse_base(&mut self, env: &NamesEnv) -> PResult<CoreIndex> {
        if let Ok(span) = self.eat_keyword("Type") {
            Ok(self.store(CorePrismExpr::Type, span))
        } else if let Ok(_) = self.eat_paren_open("(") {
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
                self.db.push_error(UnknownName {
                    span: found_name_span,
                });
                Ok(self.store(CorePrismExpr::Free, found_name_span))
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
