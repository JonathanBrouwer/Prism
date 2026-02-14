use crate::lang::PrismDb;
use prism_diag::IntoDiag;

#[derive(Copy, Clone)]
pub struct ErrorGuaranteed(());

impl PrismDb {
    pub fn push_error(&mut self, diag: impl IntoDiag<PrismDb>) -> ErrorGuaranteed {
        let diag = diag.into_diag(self);
        self.diags.push(diag);
        ErrorGuaranteed(())
    }
}
