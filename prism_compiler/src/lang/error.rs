use crate::lang::PrismDb;
use prism_diag::RenderConfig;
use std::mem;

impl PrismDb {
    pub fn eprint_errors(&mut self) {
        let errors = mem::take(&mut self.errors);
        for error in errors {
            eprintln!(
                "{}\n",
                error.render(&RenderConfig::default(), &self.input.inner())
            );
        }
    }

    pub fn assert_no_errors(&mut self) {
        if !self.errors.is_empty() {
            self.eprint_errors();
            panic!("Errors encountered, see above");
        }
    }
}
