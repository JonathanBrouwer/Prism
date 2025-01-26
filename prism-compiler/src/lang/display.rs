use std::fmt::Write;

use crate::lang::UnionIndex;
use crate::lang::{PrismEnv, PrismExpr};

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

impl<'arn, 'grm: 'arn> PrismExpr<'arn, 'grm> {
    /// Returns the precedence level of a `PartialExpr`
    fn precedence_level(&self) -> PrecedenceLevel {
        match self {
            PrismExpr::Let(_, _, _) => PrecedenceLevel::Let,
            PrismExpr::FnConstruct(_, _) => PrecedenceLevel::Construct,
            PrismExpr::FnType(_, _, _) => PrecedenceLevel::FnType,
            PrismExpr::TypeAssert(_, _) => PrecedenceLevel::TypeAssert,
            PrismExpr::FnDestruct(_, _) => PrecedenceLevel::Destruct,
            PrismExpr::Free => PrecedenceLevel::Base,
            PrismExpr::Shift(_, _) => PrecedenceLevel::Base,
            PrismExpr::Type => PrecedenceLevel::Base,
            PrismExpr::DeBruijnIndex(_) => PrecedenceLevel::Base,
            PrismExpr::Name(_) => PrecedenceLevel::Base,
            PrismExpr::ShiftLabel(_, _) => PrecedenceLevel::Base,
            PrismExpr::ShiftTo(_, _, _) => PrecedenceLevel::Base,
            PrismExpr::ParserValue(_) => PrecedenceLevel::Base,
            PrismExpr::ParserValueType => PrecedenceLevel::Base,
        }
    }
}

impl<'arn, 'grm: 'arn> PrismEnv<'arn, 'grm> {
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
            PrismExpr::Type => write!(w, "Type")?,
            PrismExpr::Let(n, v, b) => {
                write!(w, "let {n} = ")?;
                self.display(v, w, PrecedenceLevel::Construct)?;
                writeln!(w, ";")?;
                self.display(b, w, PrecedenceLevel::Let)?;
            }
            PrismExpr::DeBruijnIndex(i) => write!(w, "#{i}")?,
            PrismExpr::FnType(n, a, b) => {
                write!(w, "({n}: ")?;
                self.display(a, w, PrecedenceLevel::TypeAssert)?;
                write!(w, ") -> ")?;
                self.display(b, w, PrecedenceLevel::FnType)?;
            }
            PrismExpr::FnConstruct(n, b) => {
                write!(w, "{n}")?;
                write!(w, "=> ")?;
                self.display(b, w, PrecedenceLevel::Construct)?;
            }
            PrismExpr::FnDestruct(a, b) => {
                self.display(a, w, PrecedenceLevel::Destruct)?;
                write!(w, " ")?;
                self.display(b, w, PrecedenceLevel::Base)?;
            }
            PrismExpr::Free => write!(w, "{{{}}}", i.0)?,
            PrismExpr::Shift(v, i) => {
                write!(w, "([SHIFT {i}] ")?;
                self.display(v, w, PrecedenceLevel::default())?;
                write!(w, ")")?;
            }
            PrismExpr::TypeAssert(e, typ) => {
                self.display(e, w, PrecedenceLevel::Destruct)?;
                write!(w, ": ")?;
                self.display(typ, w, PrecedenceLevel::Destruct)?;
            }
            PrismExpr::Name(n) => {
                write!(w, "{n}")?;
            }
            PrismExpr::ShiftLabel(v, g) => {
                write!(w, "([SHIFT POINT {g:?}] ")?;
                self.display(v, w, max_precedence)?;
                write!(w, ")")?;
            }
            PrismExpr::ShiftTo(v, g, _) => {
                write!(w, "([SHIFT TO {g:?}] ")?;
                self.display(v, w, max_precedence)?;
                write!(w, ")")?;
            }
            PrismExpr::ParserValue(_) => {
                write!(w, "[PARSER VALUE]")?;
            }
            PrismExpr::ParserValueType => {
                write!(w, "Parsed")?;
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
