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
            &Expr::Let {
                name,
                value: v,
                body: b,
            } => {
                let name = name.map(|name| self.input.slice(name)).unwrap_or("_");
                write!(w, "let {name} = ")?;
                self.display(v, w, PrecedenceLevel::Construct)?;
                writeln!(w, ";")?;
                self.display(b, w, PrecedenceLevel::Let)?;
            }
            &Expr::DeBruijnIndex { idx: i } => write!(w, "#{i}")?,
            &Expr::FnType {
                arg_name,
                arg_type: a,
                body: b,
            } => {
                if let Some(arg_name) = arg_name {
                    let arg_name = self.input.slice(arg_name);
                    write!(w, "({arg_name}: ")?;
                }
                self.display(a, w, PrecedenceLevel::TypeAssert)?;
                if let Some(_) = arg_name {
                    write!(w, ")")?;
                }
                write!(w, " -> ")?;
                self.display(b, w, PrecedenceLevel::FnType)?;
            }
            &Expr::FnConstruct {
                arg_name,
                arg_type,
                body: b,
            } => {
                let arg_name = arg_name
                    .map(|arg_name| self.input.slice(arg_name))
                    .unwrap_or("_");

                write!(w, "({arg_name}: ")?;
                self.display(arg_type, w, PrecedenceLevel::TypeAssert)?;
                write!(w, ") => ")?;
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
