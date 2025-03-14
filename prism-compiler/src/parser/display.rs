use std::fmt::Write;

use crate::lang::PrismEnv;
use crate::lang::display::PrecedenceLevel;
use crate::parser::{ParsedIndex, ParsedPrismExpr};

impl<'arn, 'grm: 'arn> ParsedPrismExpr<'arn, 'grm> {
    /// Returns the precedence level of a `PartialExpr`
    fn precedence_level(&self) -> PrecedenceLevel {
        match self {
            ParsedPrismExpr::Let(..) => PrecedenceLevel::Let,
            ParsedPrismExpr::FnConstruct(..) => PrecedenceLevel::Construct,
            ParsedPrismExpr::FnType(..) => PrecedenceLevel::FnType,
            ParsedPrismExpr::TypeAssert(..) => PrecedenceLevel::TypeAssert,
            ParsedPrismExpr::FnDestruct(..) => PrecedenceLevel::Destruct,
            ParsedPrismExpr::Free => PrecedenceLevel::Base,
            ParsedPrismExpr::Type => PrecedenceLevel::Base,
            ParsedPrismExpr::Name(..) => PrecedenceLevel::Base,
            ParsedPrismExpr::GrammarValue(..) => PrecedenceLevel::Base,
            ParsedPrismExpr::GrammarType => PrecedenceLevel::Base,
            ParsedPrismExpr::ShiftTo { .. } => PrecedenceLevel::Base,
        }
    }
}

impl<'arn, 'grm: 'arn> PrismEnv<'arn, 'grm> {
    fn parse_display(
        &self,
        i: ParsedIndex,
        w: &mut impl Write,
        max_precedence: PrecedenceLevel,
    ) -> std::fmt::Result {
        let e = self.parsed_values[*i];

        if e.precedence_level() < max_precedence {
            write!(w, "(")?;
        }

        match e {
            ParsedPrismExpr::Type => write!(w, "Type")?,
            ParsedPrismExpr::Let(n, v, b) => {
                write!(w, "let {n} = ")?;
                self.parse_display(v, w, PrecedenceLevel::Construct)?;
                writeln!(w, ";")?;
                self.parse_display(b, w, PrecedenceLevel::Let)?;
            }
            ParsedPrismExpr::Name(n) => write!(w, "{n}")?,
            ParsedPrismExpr::FnType(n, a, b) => {
                write!(w, "({n}: ")?;
                self.parse_display(a, w, PrecedenceLevel::TypeAssert)?;
                write!(w, ") -> ")?;
                self.parse_display(b, w, PrecedenceLevel::FnType)?;
            }
            ParsedPrismExpr::FnConstruct(n, b) => {
                write!(w, "{n} => ")?;
                self.parse_display(b, w, PrecedenceLevel::Construct)?;
            }
            ParsedPrismExpr::FnDestruct(a, b) => {
                self.parse_display(a, w, PrecedenceLevel::Destruct)?;
                write!(w, " ")?;
                self.parse_display(b, w, PrecedenceLevel::Base)?;
            }
            ParsedPrismExpr::Free => write!(w, "{{{}}}", i.0)?,
            ParsedPrismExpr::TypeAssert(e, typ) => {
                self.parse_display(e, w, PrecedenceLevel::Destruct)?;
                write!(w, ": ")?;
                self.parse_display(typ, w, PrecedenceLevel::Destruct)?;
            }
            ParsedPrismExpr::GrammarValue(_) => {
                write!(w, "[GRAMMAR]")?;
            }
            ParsedPrismExpr::GrammarType => {
                write!(w, "Grammar")?;
            }
            ParsedPrismExpr::ShiftTo {
                expr: v,
                id: guid,
                vars: vars,
            } => {
                writeln!(w, "[SHIFT TO {guid:?}]")?;
                for (n, v) in vars {
                    write!(w, "  * {n} = ")?;
                    if let Some(v) = v.try_into_value::<ParsedIndex>() {
                        self.parse_display(*v, w, PrecedenceLevel::Base)?;
                    } else {
                        write!(w, "{}", v.to_debug_string(self.input))?;
                    }
                    writeln!(w)?;
                }
                self.parse_display(v, w, PrecedenceLevel::default())?;
            }
        }

        if e.precedence_level() < max_precedence {
            write!(w, ")")?;
        }

        Ok(())
    }

    pub fn parse_index_to_string(&self, i: ParsedIndex) -> String {
        let mut s = String::new();
        self.parse_display(i, &mut s, PrecedenceLevel::default())
            .expect("Writing to String shouldn't fail");
        s
    }
}
