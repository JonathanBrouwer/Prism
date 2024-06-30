use std::fmt::Write;

use crate::lang::display::PrecedenceLevel::*;
use crate::lang::UnionIndex;
use crate::lang::{PartialExpr, TcEnv};

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Default)]
pub enum PrecedenceLevel {
    #[default]
    Let,
    Construct,
    FnType,
    Destruct,
    Base,
}

impl PartialExpr {
    fn precendence_level(&self) -> PrecedenceLevel {
        match self {
            PartialExpr::Type => Base,
            PartialExpr::Let(_, _) => Let,
            PartialExpr::DeBruijnIndex(_) => Base,
            PartialExpr::FnType(_, _) => FnType,
            PartialExpr::FnConstruct(_, _) => Construct,
            PartialExpr::FnDestruct(_, _) => Destruct,
            PartialExpr::Free => Base,
            PartialExpr::Shift(_, _) => Base
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
        let e = self.values[i.0];

        if e.precendence_level() < max_precedence {
            write!(w, "(")?;
        }

        match e {
            PartialExpr::Type => write!(w, "Type")?,
            PartialExpr::Let(v, b) => {
                write!(w, "let ")?;
                self.display(v, w, Construct)?;
                writeln!(w, ";")?;
                self.display(b, w, Let)?;
            }
            PartialExpr::DeBruijnIndex(i) => write!(w, "#{i}")?,
            PartialExpr::FnType(a, b) => {
                self.display(a, w, Destruct)?;
                write!(w, " -> ")?;
                self.display(b, w, FnType)?;
            }
            PartialExpr::FnConstruct(a, b) => {
                self.display(a, w, FnType)?;
                write!(w, " => ")?;
                self.display(b, w, Construct)?;
            }
            PartialExpr::FnDestruct(a, b) => {
                self.display(a, w, Destruct)?;
                write!(w, " ")?;
                self.display(b, w, Base)?;
            }
            PartialExpr::Free => write!(w, "{{{}}}", i.0)?,
            PartialExpr::Shift(v, i) => {
                write!(w, "([SHIFT {i}] ")?;
                self.display(v, w, PrecedenceLevel::default())?;
                write!(w, ")")?;
            },
        }

        if e.precendence_level() < max_precedence {
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
