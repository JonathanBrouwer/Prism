use std::fmt::{Debug, Formatter};
use Term::*;

#[derive(Clone)]
pub enum Term<'src> {
    Type,
    Var {
        n: &'src str,
    },
    Let {
        n: &'src str,
        t: Box<Term<'src>>,
        v: Box<Term<'src>>,
        b: Box<Term<'src>>,
    },
    FunType {
        n: &'src str,
        at: Box<Term<'src>>,
        bt: Box<Term<'src>>,
    },
    FunConstruct {
        n: &'src str,
        at: Box<Term<'src>>,
        b: Box<Term<'src>>,
    },
    FunDestruct {
        f: Box<Term<'src>>,
        a: Box<Term<'src>>,
    },
}

impl<'src> Term<'src> {
    pub fn subst(&mut self, nr: &'src str, vr: &Term<'src>) {
        match self {
            Type {} => {}
            Var { n } if *n == nr => *self = (*vr).clone(),
            Var { .. } => {}
            Let { n, t, v, b } => {
                t.subst(nr, vr);
                v.subst(nr, vr);
                if *n == nr {
                    return;
                }
                b.subst(nr, vr);
            }
            FunType { n, at, bt } => {
                at.subst(nr, vr);
                if *n == nr {
                    return;
                }
                bt.subst(nr, vr);
            }
            FunConstruct { n, at, b } => {
                at.subst(nr, vr);
                if *n == nr {
                    return;
                }
                b.subst(nr, vr);
            }
            FunDestruct { f, a } => {
                f.subst(nr, vr);
                a.subst(nr, vr);
            }
        }
    }
    pub fn subst_self(mut self, nr: &'src str, vr: &Term<'src>) -> Self {
        self.subst(nr, vr);
        self
    }
}

impl<'src> Debug for Term<'src> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Type => write!(fmt, "Type"),
            Var { n } => write!(fmt, "{}", n),
            Let { n, t, v, b } => {
                write!(fmt, "let {} : {:?} = {:?}\n{:?}", n, t, v, b)
            }
            FunType { n, at, bt } => write!(fmt, "({}:{:?})->({:?})", n, at, bt),
            FunConstruct { n, at, b } => write!(fmt, "/{}:({:?}).({:?})", n, at, b),
            FunDestruct { f, a } => write!(fmt, "({:?} {:?})", f, a),
        }
    }
}
