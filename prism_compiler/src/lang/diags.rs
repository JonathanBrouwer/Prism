use crate::args::ErrorFormat;
use crate::lang::PrismDb;
use prism_diag::{Diag, IntoDiag, RenderConfig, RenderFormat};
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::mem;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct ErrorGuaranteed(());

#[derive(Copy, Clone)]
pub struct DiagPoint(usize);

impl PrismDb {
    pub fn push_error(&mut self, diag: impl IntoDiag<PrismDb>) -> ErrorGuaranteed {
        let diag = diag.into_diag(self);
        self.diags.push(diag);
        ErrorGuaranteed(())
    }

    pub fn point(&self) -> DiagPoint {
        DiagPoint(self.diags.len())
    }

    pub fn has_errored(&self, token: DiagPoint) -> Result<(), ErrorGuaranteed> {
        match self.diags.len().cmp(&token.0) {
            Ordering::Less => unreachable!(),
            Ordering::Equal => Ok(()),
            Ordering::Greater => Err(ErrorGuaranteed(())),
        }
    }

    pub fn assert_has_errored(&self) -> ErrorGuaranteed {
        assert!(!self.diags.is_empty(), "Has errored");
        ErrorGuaranteed(())
    }

    pub fn reset_diags_to_point(&mut self, point: DiagPoint) {
        self.diags.truncate(point.0);
    }

    pub fn take_diags(&mut self) -> PrismDiags<'_> {
        let diags = mem::take(&mut self.diags);
        PrismDiags { db: self, diags }
    }
}

pub struct PrismDiags<'a> {
    db: &'a PrismDb,
    diags: Vec<Diag>,
}

impl PrismDiags<'_> {
    pub fn has_errored(&self) -> Result<(), ErrorGuaranteed> {
        if self.diags.is_empty() {
            Ok(())
        } else {
            Err(ErrorGuaranteed(()))
        }
    }
}

impl Display for PrismDiags<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let render_config = match self.db.args.error_format {
            ErrorFormat::Pretty => RenderConfig {
                format: RenderFormat::Styled,
            },
            ErrorFormat::Plain => RenderConfig {
                format: RenderFormat::Plain,
            },
        };

        for diag in &self.diags {
            writeln!(f, "{}\n", diag.render(&render_config, &self.db.input))?;
        }
        Ok(())
    }
}
