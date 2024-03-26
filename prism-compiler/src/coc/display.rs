use std::fmt::Write;

use crate::coc::{PartialExpr, TcEnv};
use crate::union_find::UnionIndex;

impl TcEnv {
    pub fn display(&mut self, i: UnionIndex, w: &mut impl Write) -> std::fmt::Result {
        let i = self.uf.find(i);

        match self.uf_values[i.0] {
            PartialExpr::Type => write!(w, "Type")?,
            PartialExpr::Let(v, b) => {
                write!(w, "let ")?;
                self.display(v, w)?;
                writeln!(w, ";")?;
                self.display(b, w)?;
            }
            PartialExpr::Var(i) => write!(w, "#{i}")?,
            PartialExpr::FnType(a, b) => {
                write!(w, "(")?;
                self.display(a, w)?;
                write!(w, ") -> (")?;
                self.display(b, w)?;
                write!(w, ")")?;
            }
            PartialExpr::FnConstruct(a, b) => {
                write!(w, "(")?;
                self.display(a, w)?;
                write!(w, ") => (")?;
                self.display(b, w)?;
                write!(w, ")")?;
            }
            PartialExpr::FnDestruct(a, b) => {
                write!(w, "(")?;
                self.display(a, w)?;
                write!(w, ") (")?;
                self.display(b, w)?;
                write!(w, ")")?;
            }
            PartialExpr::Free => write!(w, "_")?,
            PartialExpr::Shift(b, i) => {
                self.display(b, w)?;
                write!(w, " [SHIFT {i}]")?;
            }
        }
        Ok(())
    }

    pub fn index_to_sm_string(&mut self, i: UnionIndex) -> String {
        //TODO sm
        let mut s = String::new();
        self.display(i, &mut s)
            .expect("Writing to String shouldn't fail");
        s
    }

    pub fn index_to_br_string(&mut self, i: UnionIndex) -> String {
        let i = self.beta_reduce(i);
        let mut s = String::new();
        self.display(i, &mut s)
            .expect("Writing to String shouldn't fail");
        s
    }
}
