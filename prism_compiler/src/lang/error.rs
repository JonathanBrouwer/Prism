use crate::args::ErrorFormat;
use crate::lang::PrismDb;
use prism_diag::{Diag, RenderConfig, RenderFormat};
use prism_input::input_table::InputTable;
use serde::{Deserialize, Serialize};
use std::mem;
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
pub struct SerializedErrors {
    pub errors: Vec<Diag>,
    pub input_table: Arc<InputTable>,
}

impl PrismDb {
    pub fn eprint_errors(&mut self) {
        let errors = mem::take(&mut self.errors);

        match self.args.error_format {
            ErrorFormat::Pretty => self.render_errors(
                errors,
                RenderConfig {
                    format: RenderFormat::Styled,
                },
            ),
            ErrorFormat::Plain => self.render_errors(
                errors,
                RenderConfig {
                    format: RenderFormat::Plain,
                },
            ),
            ErrorFormat::Json => {
                let errors = SerializedErrors {
                    errors,
                    input_table: self.input.clone(),
                };

                eprintln!("{}", serde_json::to_string_pretty(&errors).unwrap());
            }
        }
    }

    fn render_errors(&self, errors: Vec<Diag>, render_config: RenderConfig) {
        for error in errors {
            eprintln!("{}\n", error.render(&render_config, &self.input.inner()));
        }
    }

    pub fn assert_no_errors(&mut self) {
        if !self.errors.is_empty() {
            self.eprint_errors();
            panic!("Errors encountered, see above");
        }
    }
}
