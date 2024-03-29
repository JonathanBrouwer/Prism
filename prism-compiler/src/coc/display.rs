use std::fmt::Write;

use crate::coc::{PartialExpr, TcEnv};
use crate::coc::display::PrecedenceLevel::*;
use crate::union_find::UnionIndex;

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Default)]
enum PrecedenceLevel {
    #[default]
    Let,
    Construct,
    Destruct,
    Base,
}

impl PartialExpr {
    fn precendence_level(&self) -> PrecedenceLevel {
        match self {
            PartialExpr::Type => Base,
            PartialExpr::Let(_, _) => Let,
            PartialExpr::Var(_) => Base,
            PartialExpr::FnType(_, _) => Construct,
            PartialExpr::FnConstruct(_, _) => Construct,
            PartialExpr::FnDestruct(_, _) => Destruct,
            PartialExpr::Free => Base,
            PartialExpr::Shift(_, _) => unreachable!(),
        }
    }
}

impl TcEnv {
    fn display(&mut self, i: UnionIndex, w: &mut impl Write, max_precedence: PrecedenceLevel) -> std::fmt::Result {
        let i = self.uf.find(i);

        let e = self.uf_values[i.0];

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
            PartialExpr::Var(i) => write!(w, "#{i}")?,
            PartialExpr::FnType(a, b) => {
                self.display(a, w, Destruct)?;
                write!(w, " -> ")?;
                self.display(b, w, Construct)?;
            }
            PartialExpr::FnConstruct(a, b) => {
                self.display(a, w, Destruct)?;
                write!(w, " => ")?;
                self.display(b, w, Construct)?;
            }
            PartialExpr::FnDestruct(a, b) => {
                self.display(a, w, Destruct)?;
                write!(w, " ")?;
                self.display(b, w, Base)?;
            }
            PartialExpr::Free => write!(w, "_")?,
            PartialExpr::Shift(..) => unreachable!(),
        }

        if e.precendence_level() < max_precedence {
            write!(w, ")")?;
        }

        Ok(())
    }

    pub fn index_to_sm_string(&mut self, i: UnionIndex) -> String {
        let i = self.simplify(i);
        let mut s = String::new();
        self.display(i, &mut s, PrecedenceLevel::default())
            .expect("Writing to String shouldn't fail");
        s
    }

    pub fn index_to_br_string(&mut self, i: UnionIndex) -> String {
        let i = self.beta_reduce(i);
        let mut s = String::new();
        self.display(i, &mut s, PrecedenceLevel::default())
            .expect("Writing to String shouldn't fail");
        s
    }
}
