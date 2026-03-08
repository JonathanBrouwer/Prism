use crate::lang::PrismDb;
use prism_diag::IntoDiag;
use std::cmp::Ordering;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct ErrorGuaranteed(());

pub struct RecoveryToken(usize);

impl PrismDb {
    pub fn push_error(&mut self, diag: impl IntoDiag<PrismDb>) -> ErrorGuaranteed {
        let diag = diag.into_diag(self);
        self.diags.push(diag);
        ErrorGuaranteed(())
    }

    pub fn recovery_point(&self) -> RecoveryToken {
        RecoveryToken(self.diags.len())
    }

    pub fn try_recover(&self, token: RecoveryToken) -> Option<ErrorGuaranteed> {
        match self.diags.len().cmp(&token.0) {
            Ordering::Less => unreachable!(),
            Ordering::Equal => None,
            Ordering::Greater => Some(ErrorGuaranteed(())),
        }
    }
}
