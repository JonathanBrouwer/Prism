use crate::args::ErrorFormat;
use crate::lang::PrismDb;
use prism_diag::{RenderConfig, RenderFormat};
use std::mem;

impl PrismDb {
    pub fn eprint_errors(&mut self) {
        let errors = mem::take(&mut self.diags);

        let render_config = match self.args.error_format {
            ErrorFormat::Pretty => RenderConfig {
                format: RenderFormat::Styled,
            },
            ErrorFormat::Plain => RenderConfig {
                format: RenderFormat::Plain,
            },
        };

        for error in errors {
            eprintln!("{}\n", error.render(&render_config, &self.input.inner()));
        }
    }

    pub fn assert_no_errors(&mut self) {
        if !self.diags.is_empty() {
            self.eprint_errors();
            panic!("Errors encountered, see above");
        }
    }
}
