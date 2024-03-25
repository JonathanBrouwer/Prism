use std::collections::HashMap;
use std::fmt::Write;

use crate::coc::{PartialExpr, TcEnv};
use crate::coc::env::{Env, UniqueVariableId};
use crate::coc::env::EnvEntry::*;
use crate::union_find::UnionIndex;

impl TcEnv {
    pub fn display(
        &mut self,
        i: UnionIndex,
        mut env: Env,
        depth: usize,
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
                self.display(v, env.clone(), depth, w, br)?;
                writeln!(w, ";")?;
                self.display(b, env.cons(RSubst(v, env.clone())), depth, w, br)?;
            }
            PartialExpr::Var(i) => {
                //TODO for non-bh, let also needs a unique id.
                //TODO then  `depth - unique id - 1 for each var?`
                write!(w, "#{i}", )?
            },
            PartialExpr::FnType(a, b) => {
                write!(w, "(")?;
                self.display(a, env.clone(), depth, w, br)?;
                write!(w, ") -> (")?;
                let id = self.new_tc_id();
                self.display(b, env.cons(RType(id)), depth + 1, w, br)?;
                write!(w, ")")?;
            }
            PartialExpr::FnConstruct(a, b) => {
                write!(w, "(")?;
                self.display(a, env.clone(), depth, w, br)?;
                write!(w, ") => (")?;
                let id = self.new_tc_id();
                self.display(b, env.cons(RType(id)), depth + 1, w, br)?;
                write!(w, ")")?;
            }
            PartialExpr::FnDestruct(a, b) => {
                write!(w, "(")?;
                self.display(a, env.clone(), depth, w, br)?;
                write!(w, ") (")?;
                self.display(b, env.clone(), depth, w, br)?;
                write!(w, ")")?;
            }
            PartialExpr::Free => write!(w, "_")?,
            PartialExpr::Shift(b, i) => {
                self.display(b, env.shift(i), depth, w, br)?;
                write!(w, " [SHIFT {i}]")?;
            }
            PartialExpr::Subst(b, (v, ref vs)) => {
                self.display(b, env.cons(RSubst(v, vs.clone())), depth, w, br)?;
            }
        }
        Ok(())
    }

    pub fn index_to_string(&mut self, i: UnionIndex, br: bool) -> String {
        let mut s = String::new();
        self.display(i, Env::new(), 0, &mut s, br).expect("Writing to String shouldn't fail");
        s
    }
}
