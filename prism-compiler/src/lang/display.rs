use std::fmt::Write;

use crate::lang::CoreIndex;
use crate::lang::env::DbEnv;
use crate::lang::{CorePrismExpr, PrismEnv};

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

impl CorePrismExpr {
    /// Returns the precedence level of a `PartialExpr`
    fn precedence_level(&self) -> PrecedenceLevel {
        match self {
            CorePrismExpr::Let(..) => PrecedenceLevel::Let,
            CorePrismExpr::FnConstruct(..) => PrecedenceLevel::Construct,
            CorePrismExpr::FnType(..) => PrecedenceLevel::FnType,
            CorePrismExpr::TypeAssert(..) => PrecedenceLevel::TypeAssert,
            CorePrismExpr::FnDestruct(..) => PrecedenceLevel::Destruct,
            CorePrismExpr::Free => PrecedenceLevel::Base,
            CorePrismExpr::Shift(..) => PrecedenceLevel::Base,
            CorePrismExpr::Type => PrecedenceLevel::Base,
            CorePrismExpr::DeBruijnIndex(..) => PrecedenceLevel::Base,
            CorePrismExpr::GrammarValue(..) => PrecedenceLevel::Base,
            CorePrismExpr::GrammarType => PrecedenceLevel::Base,
        }
    }
}

impl PrismEnv {
    fn display(
        &self,
        i: CoreIndex,
        w: &mut impl Write,
        max_precedence: PrecedenceLevel,
    ) -> std::fmt::Result {
        let e = &self.checked_values[*i];

        if e.precedence_level() < max_precedence {
            write!(w, "(")?;
        }

        match e {
            CorePrismExpr::Type => write!(w, "Type")?,
            &CorePrismExpr::Let(v, b) => {
                write!(w, "let ")?;
                self.display(v, w, PrecedenceLevel::Construct)?;
                writeln!(w, ";")?;
                self.display(b, w, PrecedenceLevel::Let)?;
            }
            &CorePrismExpr::DeBruijnIndex(i) => write!(w, "#{i}")?,
            &CorePrismExpr::FnType(a, b) => {
                self.display(a, w, PrecedenceLevel::TypeAssert)?;
                write!(w, " -> ")?;
                self.display(b, w, PrecedenceLevel::FnType)?;
            }
            &CorePrismExpr::FnConstruct(b) => {
                write!(w, "=> ")?;
                self.display(b, w, PrecedenceLevel::Construct)?;
            }
            &CorePrismExpr::FnDestruct(a, b) => {
                self.display(a, w, PrecedenceLevel::Destruct)?;
                write!(w, " ")?;
                self.display(b, w, PrecedenceLevel::Base)?;
            }
            CorePrismExpr::Free => write!(w, "{{{}}}", i.0)?,
            &CorePrismExpr::Shift(v, i) => {
                write!(w, "([SHIFT {i}] ")?;
                self.display(v, w, PrecedenceLevel::default())?;
                write!(w, ")")?;
            }
            &CorePrismExpr::TypeAssert(e, typ) => {
                self.display(e, w, PrecedenceLevel::Destruct)?;
                write!(w, ": ")?;
                self.display(typ, w, PrecedenceLevel::Destruct)?;
            }
            CorePrismExpr::GrammarValue(_) => {
                write!(w, "[GRAMMAR]")?;
            }
            CorePrismExpr::GrammarType => {
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
