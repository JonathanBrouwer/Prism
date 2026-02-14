mod layout;

use crate::lang::diags::ErrorGuaranteed;
use crate::lang::{CoreIndex, CorePrismExpr, PrismDb, ValueOrigin};
use prism_diag::{Annotation, AnnotationGroup, Diag};
use prism_diag_derive::Diagnostic;
use prism_input::input_table::InputTableIndex;
use prism_input::pos::Pos;
use prism_input::span::Span;
use prism_input::tokens::{Token, TokenType, Tokens};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
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
        let mut parse_env = ParserPrismEnv::new(self, file);
        parse_env.parse_file()
    }
}

struct ParserPrismEnv<'a> {
    db: &'a mut PrismDb,
    file: InputTableIndex,
    pos: Pos,
    tokens: Vec<Token>,

    fail_pos: Pos,
    expected: Vec<(Span, Expected)>,
}

type PResult<T> = Result<T, ExpectedGuaranteed>;

pub enum Expected {
    Literal(String),
}

impl Display for Expected {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Expected::Literal(exp) => write!(f, "\"{exp}\""),
        }
    }
}

struct ExpectedGuaranteed(());

impl<'a> ParserPrismEnv<'a> {
    pub fn new(db: &'a mut PrismDb, file: InputTableIndex) -> Self {
        let pos = db.input.inner().start_of(file);
        Self {
            file,
            pos,
            tokens: vec![],
            db,

            fail_pos: pos,
            expected: vec![],
        }
    }

    fn expect(&mut self, span: Span, expected: Expected) -> ExpectedGuaranteed {
        match span.end_pos().cmp(&self.fail_pos) {
            Ordering::Less => {
                assert!(!self.expected.is_empty());
            }
            Ordering::Equal => {
                self.expected.push((span, expected));
            }
            Ordering::Greater => {
                self.expected.clear();
                self.expected.push((span, expected));
            }
        }
        ExpectedGuaranteed(())
    }

    fn expected_into_diag(&mut self) -> Diag {
        assert!(!self.expected.is_empty());

        let mut labels_map: HashMap<Pos, (Pos, Vec<_>)> = HashMap::new();
        for (span, exp) in &self.expected {
            let (end_pos, expected) = labels_map
                .entry(span.start_pos())
                .or_insert((span.end_pos(), Vec::new()));
            *end_pos = (*end_pos).max(span.end_pos());
            expected.push(exp);
        }

        Diag {
            title: "Parsing failed".into(),
            id: "parser".into(),
            groups: vec![AnnotationGroup {
                annotations: labels_map
                    .into_iter()
                    .map(|(start, (end, labels))| Annotation {
                        span: start.span_to(end),
                        label: Some(match &labels[..] {
                            [] => unreachable!(),
                            [label] => format!("Expected: {}", label),
                            ref labels => format!(
                                "Expected one of: {}",
                                labels
                                    .iter()
                                    .map(|v| v.to_string())
                                    .collect::<Vec<_>>()
                                    .join(" ")
                            ),
                        }),
                    })
                    .collect(),
            }],
        }
    }

    pub fn parse_file(mut self) -> (CoreIndex, Arc<Tokens>) {
        let start = self.pos;

        _ = self.parse_program();
        self.eat_layout();
        // assert!(self.pos.next(&self.db.input).is_none());

        let free = self.db.store(
            CorePrismExpr::Type,
            ValueOrigin::SourceCode(start.span_to(start)),
        );

        if !self.expected.is_empty() {
            let diag = self.expected_into_diag();
            self.db.push_error(diag);
        }

        (free, Arc::new(Tokens(self.tokens)))
        // let mut parsables = HashMap::new();

        // parsables.insert("Expr", ParsableDyn::new::<ParsedIndex>());
        //
        // let (expr, tokens, errs) = run_parser_rule::<ParserPrismEnv<'a>, ParsedIndex, SetError>(
        //     &GRAMMAR.1,
        //     "expr",
        //     self.db.input.clone(),
        //     file,
        //     parsables,
        //     self,
        // );
        //
        // for err in errs.errors {
        //     self.db.diags.push(err.diag());
        // }
        //
        // (*expr, tokens)
    }

    fn parse_program(&mut self) -> PResult<()> {
        self.eat_lit("program")?;
        Ok(())
    }

    fn eat_char(&mut self) -> Result<char, ()> {
        match self.pos.next(&self.db.input) {
            None => Err(()),
            Some((ch, next_pos)) => {
                self.pos = next_pos;
                Ok(ch)
            }
        }
    }

    pub fn peek_lit_raw(&mut self, lit: &str) -> Result<Span, Span> {
        let start = self.pos;
        for expected_char in lit.chars() {
            let Ok(actual_char) = self.eat_char() else {
                let fail_pos = self.pos;
                self.pos = start;
                return Err(start.span_to(fail_pos));
            };
            if expected_char != actual_char {
                let fail_pos = self.pos;
                self.pos = start;
                return Err(start.span_to(fail_pos));
            }
        }
        Ok(start.span_to(self.pos))
    }

    pub fn eat_lit_raw(&mut self, lit: &str) -> PResult<Span> {
        match self.peek_lit_raw(lit) {
            Ok(res) => Ok(res),
            Err(err_span) => Err(self.expect(err_span, Expected::Literal(lit.to_string()))),
        }
    }

    pub fn peek_lit(&mut self, lit: &str) -> Result<Span, Span> {
        self.eat_layout();
        self.peek_lit_raw(lit)
    }

    pub fn eat_lit(&mut self, lit: &str) -> PResult<Span> {
        self.eat_layout();
        self.eat_lit_raw(lit)
    }
}
