use crate::coc::env::EnvEntry::*;
use crate::coc::env::{Env, EnvEntry};
use crate::coc::{PartialExpr, TcEnv};
use crate::union_find::UnionIndex;
use std::fmt::Write;

impl TcEnv {
    pub fn display(
        &mut self,
        i: UnionIndex,
        mut env: Env,
        w: &mut impl Write,
        br: bool,
    ) -> std::fmt::Result {
        let mut i = self.uf.find(i);
        if br {
            (i, env) = self.brh(i, env.clone());
            i = self.uf.find(i);
        }

        match self.uf_values[i.0] {
            PartialExpr::Type => write!(w, "Type")?,
            PartialExpr::Let(v, b) => {
                write!(w, "let ")?;
                self.display(v, env.clone(), w, br)?;
                writeln!(w, ";")?;
                self.display(b, env.cons(RSubst(v, env.clone())), w, br)?;
            }
            PartialExpr::Var(i) => write!(w, "#{i}")?,
            PartialExpr::FnType(a, b) => {
                write!(w, "(")?;
                self.display(a, env.clone(), w, br)?;
                write!(w, ") -> (")?;
                self.display(b, env.cons(EnvEntry::RType), w, br)?;
                write!(w, ")")?;
            }
            PartialExpr::FnConstruct(a, b) => {
                write!(w, "(")?;
                self.display(a, env.clone(), w, br)?;
                write!(w, ") => (")?;
                self.display(b, env.cons(EnvEntry::RType), w, br)?;
                write!(w, ")")?;
            }
            PartialExpr::FnDestruct(a, b) => {
                write!(w, "(")?;
                self.display(a, env.clone(), w, br)?;
                write!(w, ") (")?;
                self.display(b, env.clone(), w, br)?;
                write!(w, ")")?;
            }
            PartialExpr::Free => write!(w, "_")?,
            PartialExpr::Shift(b, i) => {
                self.display(b, env.shift(i), w, br)?;
            }
            PartialExpr::Subst(b, (v, ref vs)) => {
                self.display(b, env.cons(RSubst(v, vs.clone())), w, br)?;
            }
        }
        Ok(())
    }

    pub fn index_to_string(&mut self, i: UnionIndex, br: bool) -> Result<String, std::fmt::Error> {
        let mut s = String::new();
        self.display(i, Env::new(), &mut s, br)?;
        Ok(s)
    }
}
