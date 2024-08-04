use std::fmt::Write;

use crate::lang::UnionIndex;
use crate::lang::{PartialExpr, TcEnv};

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

impl PartialExpr {
    /// Returns the precedence level of a `PartialExpr`
    fn precedence_level(&self) -> PrecedenceLevel {
        match self {
            PartialExpr::Let(_, _) => PrecedenceLevel::Let,
            PartialExpr::FnConstruct(_) => PrecedenceLevel::Construct,
            PartialExpr::FnType(_, _) => PrecedenceLevel::FnType,
            PartialExpr::TypeAssert(_, _) => PrecedenceLevel::TypeAssert,
            PartialExpr::FnDestruct(_, _) => PrecedenceLevel::Destruct,
            PartialExpr::Free => PrecedenceLevel::Base,
            PartialExpr::Shift(_, _) => PrecedenceLevel::Base,
            PartialExpr::Type => PrecedenceLevel::Base,
            PartialExpr::DeBruijnIndex(_) => PrecedenceLevel::Base,
        }
    }
}

impl TcEnv {
    fn display(
        &self,
        i: UnionIndex,
        w: &mut impl Write,
        max_precedence: PrecedenceLevel,
    ) -> std::fmt::Result {
        let e = self.values[*i];

        if e.precedence_level() < max_precedence {
            write!(w, "(")?;
        }

        match e {
            PartialExpr::Type => write!(w, "Type")?,
            PartialExpr::Let(v, b) => {
                write!(w, "let ")?;
                self.display(v, w, PrecedenceLevel::Construct)?;
                writeln!(w, ";")?;
                self.display(b, w, PrecedenceLevel::Let)?;
            }
            PartialExpr::DeBruijnIndex(i) => write!(w, "#{i}")?,
            PartialExpr::FnType(a, b) => {
                self.display(a, w, PrecedenceLevel::TypeAssert)?;
                write!(w, " -> ")?;
                self.display(b, w, PrecedenceLevel::FnType)?;
            }
            PartialExpr::FnConstruct(b) => {
                write!(w, "=> ")?;
                self.display(b, w, PrecedenceLevel::Construct)?;
            }
            PartialExpr::FnDestruct(a, b) => {
                self.display(a, w, PrecedenceLevel::Destruct)?;
                write!(w, " ")?;
                self.display(b, w, PrecedenceLevel::Base)?;
            }
            PartialExpr::Free => write!(w, "{{{}}}", i.0)?,
            PartialExpr::Shift(v, i) => {
                write!(w, "([SHIFT {i}] ")?;
                self.display(v, w, PrecedenceLevel::default())?;
                write!(w, ")")?;
            }
            PartialExpr::TypeAssert(e, typ) => {
                self.display(e, w, PrecedenceLevel::Destruct)?;
                write!(w, ": ")?;
                self.display(typ, w, PrecedenceLevel::Destruct)?;
            }
        }

        if e.precedence_level() < max_precedence {
            write!(w, ")")?;
        }

        Ok(())
    }

    pub fn index_to_string(&self, i: UnionIndex) -> String {
        let mut s = String::new();
        self.display(i, &mut s, PrecedenceLevel::default())
            .expect("Writing to String shouldn't fail");
        s
    }

    pub fn index_to_sm_string(&mut self, i: UnionIndex) -> String {
        let i = self.simplify(i);
        self.index_to_string(i)
    }

    pub fn index_to_br_string(&mut self, i: UnionIndex) -> String {
        let i = self.beta_reduce(i);
        self.index_to_string(i)
    }
}
