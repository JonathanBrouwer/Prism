use crate::lang::{CoreIndex, PrismDb};
use prism_diag_derive::Diagnostic;
use prism_input::input_table::InputTableIndex;
use prism_input::tokens::Tokens;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;

impl PrismDb {
    pub fn load_file(&mut self, path: PathBuf) -> Option<InputTableIndex> {
        #[derive(Diagnostic)]
        #[diag(title = format!("Failed to read file `{:?}`: {}", self.path, self.error))]
        struct FailedToRead {
            path: PathBuf,
            error: io::Error,
        }

        match std::fs::read_to_string(&path) {
            Ok(program) => Some(self.load_input(program, path)),
            Err(error) => {
                self.push_error(FailedToRead { path, error });
                None
            }
        }
    }

    pub fn load_input(&mut self, data: String, path: PathBuf) -> InputTableIndex {
        self.input.inner_mut().get_or_push_file(data, path)
    }

    pub fn parse_prism_file(&mut self, file: InputTableIndex) -> (CoreIndex, Arc<Tokens>) {
        let mut parse_env = ParserPrismEnv::new(self);
        parse_env.parse_file(file)
    }
}

pub struct ParserPrismEnv<'a> {
    db: &'a mut PrismDb,
}

impl<'a> ParserPrismEnv<'a> {
    pub fn new(db: &'a mut PrismDb) -> Self {
        Self { db }
    }

    pub fn parse_file(&mut self, file: InputTableIndex) -> (CoreIndex, Arc<Tokens>) {
        // self.db.store()
        todo!()
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
}
