use std::fmt::Write;

use crate::lang::CoreIndex;
use crate::lang::env::DbEnv;
use crate::lang::{Expr, PrismDb};

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

impl Expr {
    /// Returns the precedence level of a `PartialExpr`
    fn precedence_level(&self) -> PrecedenceLevel {
        match self {
            Expr::Let { .. } => PrecedenceLevel::Let,
            Expr::FnConstruct { .. } => PrecedenceLevel::Construct,
            Expr::FnType { .. } => PrecedenceLevel::FnType,
            Expr::TypeAssert { .. } => PrecedenceLevel::TypeAssert,
            Expr::FnDestruct { .. } => PrecedenceLevel::Destruct,
            Expr::Free => PrecedenceLevel::Base,
            Expr::Shift(..) => PrecedenceLevel::Base,
            Expr::Type => PrecedenceLevel::Base,
            Expr::DeBruijnIndex { .. } => PrecedenceLevel::Base,
            Expr::GrammarValue(..) => PrecedenceLevel::Base,
            Expr::GrammarType => PrecedenceLevel::Base,
        }
    }
}

impl PrismDb {
    fn display(
        &self,
        i: CoreIndex,
        w: &mut impl Write,
        max_precedence: PrecedenceLevel,
    ) -> std::fmt::Result {
        let e = &self.exprs[*i];

        if e.precedence_level() < max_precedence {
            write!(w, "(")?;
        }

        match e {
            Expr::Type => write!(w, "Type")?,
            &Expr::Let { value: v, body: b } => {
                write!(w, "let _ = ")?;
                self.display(v, w, PrecedenceLevel::Construct)?;
                writeln!(w, ";")?;
                self.display(b, w, PrecedenceLevel::Let)?;
            }
            &Expr::DeBruijnIndex { idx: i } => write!(w, "#{i}")?,
            &Expr::FnType {
                arg_type: a,
                body: b,
            } => {
                self.display(a, w, PrecedenceLevel::TypeAssert)?;
                write!(w, " -> ")?;
                self.display(b, w, PrecedenceLevel::FnType)?;
            }
            &Expr::FnConstruct { body: b } => {
                write!(w, "_ => ")?;
                self.display(b, w, PrecedenceLevel::Construct)?;
            }
            &Expr::FnDestruct {
                function: a,
                arg: b,
            } => {
                self.display(a, w, PrecedenceLevel::Destruct)?;
                write!(w, " ")?;
                self.display(b, w, PrecedenceLevel::Base)?;
            }
            Expr::Free => write!(w, "_")?,
            &Expr::Shift(v, i) => {
                write!(w, "([SHIFT {i}] ")?;
                self.display(v, w, PrecedenceLevel::default())?;
                write!(w, ")")?;
            }
            &Expr::TypeAssert {
                value: e,
                type_hint: typ,
            } => {
                self.display(e, w, PrecedenceLevel::Destruct)?;
                write!(w, ": ")?;
                self.display(typ, w, PrecedenceLevel::Destruct)?;
            }
            Expr::GrammarValue(_) => {
                write!(w, "[GRAMMAR]")?;
            }
            Expr::GrammarType => {
                write!(w, "Grammar")?;
            }
        }

        if e.precedence_level() < max_precedence {
            write!(w, ")")?;
        }

        Ok(())
    }

    pub fn index_to_string(&self, i: CoreIndex) -> String {
        let mut s = String::new();
        self.display(i, &mut s, PrecedenceLevel::default())
            .expect("Writing to String shouldn't fail");
        s
    }

    pub fn index_to_sm_string(&mut self, i: CoreIndex) -> String {
        let i = self.simplify(i);
        self.index_to_string(i)
    }

    pub fn index_to_br_string(&mut self, i: CoreIndex, env: &DbEnv) -> String {
        let i = self.beta_reduce(i, env);
        self.index_to_string(i)
    }
}
