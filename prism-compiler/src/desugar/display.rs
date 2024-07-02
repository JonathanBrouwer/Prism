use std::fmt::Write;
use crate::desugar::{ParseEnv, ParseIndex, SourceExpr};
use crate::lang::display::PrecedenceLevel;
use crate::lang::display::PrecedenceLevel::{Base, Construct, Destruct, FnType, Let};

impl SourceExpr {
    fn precendence_level(&self) -> PrecedenceLevel {
        match self {
            SourceExpr::Type => Base,
            SourceExpr::Let(_, _, _) => Let,
            SourceExpr::Variable(_) => Base,
            SourceExpr::FnType(_, _, _) => FnType,
            SourceExpr::FnConstruct(_, _, _) => Construct,
            SourceExpr::FnDestruct(_, _) => Destruct,
            SourceExpr::ScopeDefine(_, _) => Base,
            SourceExpr::ScopeEnter(_, _) => Base,
            SourceExpr::ScopeExit(_) => Base,
        }
    }
}

impl ParseEnv {
    fn display(
        &self,
        i: ParseIndex,
        w: &mut impl Write,
        max_precedence: PrecedenceLevel,
    ) -> std::fmt::Result {
        let e = &self.values[i.0];

        if e.precendence_level() < max_precedence {
            write!(w, "(")?;
        }

        match e {
            SourceExpr::Type => write!(w, "Type")?,
            SourceExpr::Let(n, v, b) => {
                write!(w, "let {n} = ")?;
                self.display(*v, w, Construct)?;
                writeln!(w, ";")?;
                self.display(*b, w, Let)?;
            }
            SourceExpr::Variable(i) => write!(w, "#{i}")?,
            SourceExpr::FnType(n, a, b) => {
                write!(w, "({n}: ")?;
                self.display(*a, w, Destruct)?;
                write!(w, ") -> ")?;
                self.display(*b, w, FnType)?;
            }
            SourceExpr::FnConstruct(n, a, b) => {
                write!(w, "({n}: ")?;
                self.display(*a, w, FnType)?;
                write!(w, ") => ")?;
                self.display(*b, w, Construct)?;
            }
            SourceExpr::FnDestruct(a, b) => {
                self.display(*a, w, Destruct)?;
                write!(w, " ")?;
                self.display(*b, w, Base)?;
            }
            SourceExpr::ScopeDefine(v, guid) => {
                write!(w, "([DEFINE {}] ", guid.0)?;
                self.display(*v, w, PrecedenceLevel::default())?;
                write!(w, ")")?;
            },
            SourceExpr::ScopeEnter(v, guid) => {
                write!(w, "([ENTER {}] ", guid.0)?;
                self.display(*v, w, PrecedenceLevel::default())?;
                write!(w, ")")?;
            },
            SourceExpr::ScopeExit(v) => {
                write!(w, "([EXIT] ")?;
                self.display(*v, w, PrecedenceLevel::default())?;
                write!(w, ")")?;
            }
        }

        if e.precendence_level() < max_precedence {
            write!(w, ")")?;
        }

        Ok(())
    }

    pub fn index_to_string(&self, i: ParseIndex) -> String {
        let mut s = String::new();
        self.display(i, &mut s, PrecedenceLevel::default())
            .expect("Writing to String shouldn't fail");
        s
    }
}