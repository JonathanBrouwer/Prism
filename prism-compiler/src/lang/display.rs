use std::fmt::Write;

use crate::lang::CheckedIndex;
use crate::lang::{CheckedPrismExpr, PrismEnv};

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Default)]
pub enum PrecedenceLevel {
    #[default]
    Let,
    Construct,
    FnType,
    TypeAssert,
    Destruct,
    Base,
}

impl<'arn, 'grm: 'arn> CheckedPrismExpr<'arn, 'grm> {
    /// Returns the precedence level of a `PartialExpr`
    fn precedence_level(&self) -> PrecedenceLevel {
        match self {
            CheckedPrismExpr::Let(_, _) => PrecedenceLevel::Let,
            CheckedPrismExpr::FnConstruct(_) => PrecedenceLevel::Construct,
            CheckedPrismExpr::FnType(_, _) => PrecedenceLevel::FnType,
            CheckedPrismExpr::TypeAssert(_, _) => PrecedenceLevel::TypeAssert,
            CheckedPrismExpr::FnDestruct(_, _) => PrecedenceLevel::Destruct,
            CheckedPrismExpr::Free => PrecedenceLevel::Base,
            CheckedPrismExpr::Shift(_, _) => PrecedenceLevel::Base,
            CheckedPrismExpr::Type => PrecedenceLevel::Base,
            CheckedPrismExpr::DeBruijnIndex(_) => PrecedenceLevel::Base,
            CheckedPrismExpr::ParserValue(_) => PrecedenceLevel::Base,
            CheckedPrismExpr::ParsedType => PrecedenceLevel::Base,
        }
    }
}

impl<'arn, 'grm: 'arn> PrismEnv<'arn, 'grm> {
    fn display(
        &self,
        i: CheckedIndex,
        w: &mut impl Write,
        max_precedence: PrecedenceLevel,
    ) -> std::fmt::Result {
        let e = self.checked_values[*i];

        if e.precedence_level() < max_precedence {
            write!(w, "(")?;
        }

        match e {
            CheckedPrismExpr::Type => write!(w, "Type")?,
            CheckedPrismExpr::Let(v, b) => {
                write!(w, "let ")?;
                self.display(v, w, PrecedenceLevel::Construct)?;
                writeln!(w, ";")?;
                self.display(b, w, PrecedenceLevel::Let)?;
            }
            CheckedPrismExpr::DeBruijnIndex(i) => write!(w, "#{i}")?,
            CheckedPrismExpr::FnType(a, b) => {
                self.display(a, w, PrecedenceLevel::TypeAssert)?;
                write!(w, " -> ")?;
                self.display(b, w, PrecedenceLevel::FnType)?;
            }
            CheckedPrismExpr::FnConstruct(b) => {
                write!(w, "=> ")?;
                self.display(b, w, PrecedenceLevel::Construct)?;
            }
            CheckedPrismExpr::FnDestruct(a, b) => {
                self.display(a, w, PrecedenceLevel::Destruct)?;
                write!(w, " ")?;
                self.display(b, w, PrecedenceLevel::Base)?;
            }
            CheckedPrismExpr::Free => write!(w, "{{{}}}", i.0)?,
            CheckedPrismExpr::Shift(v, i) => {
                write!(w, "([SHIFT {i}] ")?;
                self.display(v, w, PrecedenceLevel::default())?;
                write!(w, ")")?;
            }
            CheckedPrismExpr::TypeAssert(e, typ) => {
                self.display(e, w, PrecedenceLevel::Destruct)?;
                write!(w, ": ")?;
                self.display(typ, w, PrecedenceLevel::Destruct)?;
            }
            CheckedPrismExpr::ParserValue(_) => {
                write!(w, "[PARSER VALUE]")?;
            }
            CheckedPrismExpr::ParsedType => {
                write!(w, "Parsed")?;
            }
        }

        if e.precedence_level() < max_precedence {
            write!(w, ")")?;
        }

        Ok(())
    }

    pub fn index_to_string(&self, i: CheckedIndex) -> String {
        let mut s = String::new();
        self.display(i, &mut s, PrecedenceLevel::default())
            .expect("Writing to String shouldn't fail");
        s
    }

    pub fn index_to_sm_string(&mut self, i: CheckedIndex) -> String {
        let i = self.simplify(i);
        self.index_to_string(i)
    }

    pub fn index_to_br_string(&mut self, i: CheckedIndex) -> String {
        let i = self.beta_reduce(i);
        self.index_to_string(i)
    }
}
