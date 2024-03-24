use std::fmt::{Display, Formatter};
use std::fmt::Write;
use crate::coc::{PartialExpr, TcEnv};
use crate::union_find::UnionIndex;

struct TcEnvExpr<'a>(&'a TcEnv, UnionIndex);

impl Display for TcEnvExpr<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.0.uf_values[self.0.uf.find_const(self.1).0] {
            PartialExpr::Type => write!(f, "Type"),
            PartialExpr::Let(v, b) => {
                writeln!(f, "let {};", TcEnvExpr(self.0, v))?;
                write!(f, "{}", TcEnvExpr(self.0, b))
            }
            PartialExpr::Var(i) => write!(f, "#{i}"),
            PartialExpr::FnType(a, b) => write!(f, "({}) -> ({})", TcEnvExpr(self.0, a), TcEnvExpr(self.0, b)),
            PartialExpr::FnConstruct(a, b) => write!(f, "({}) => ({})", TcEnvExpr(self.0, a), TcEnvExpr(self.0, b)),
            PartialExpr::FnDestruct(a, b) => write!(f, "({}) ({})", TcEnvExpr(self.0, a), TcEnvExpr(self.0, b)),
            PartialExpr::Free => write!(f, "_"),
            PartialExpr::Shift(b, i) => write!(f, "{} [SHIFT {i}]", TcEnvExpr(self.0, b)),
            PartialExpr::Subst(b, (v, _)) => write!(f, "{} [SUBST {} ENV NOT SHOWN]", TcEnvExpr(self.0, b), TcEnvExpr(self.0, v)),
        }
    }
}


impl TcEnv {
    pub fn display(&self, i: UnionIndex, mut w: impl Write) -> std::fmt::Result {
        write!(w, "{}", TcEnvExpr(self, i))
    }

    pub fn to_string(&self, i: UnionIndex) -> Result<String, std::fmt::Error> {
        let mut s = String::new();
        self.display(i, &mut s)?;
        Ok(s)
    }
}
