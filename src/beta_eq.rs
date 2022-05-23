use crate::term::Term;
use crate::term::Term::*;

pub fn is_beta_eq(t1: Term, t2: Term) -> bool {
    //TODO quick-exit if t1 and t2 are alpha-equiv
    let t1 = beta_reduce_head(t1);
    let t2 = beta_reduce_head(t2);
    match (t1, t2) {
        (Type, Type) => true,
        (Var { n: n1 }, Var { n: n2 }) => n1 == n2,
        (
            FunType {
                n: n1,
                at: at1,
                bt: bt1,
            },
            FunType {
                n: n2,
                at: at2,
                bt: bt2,
            },
        ) => {
            let fresh_name = format!("%{}%{}%", n1, n2);
            let fresh_var = Var { n: &fresh_name };
            is_beta_eq(*at1, *at2)
                && is_beta_eq(
                    bt1.subst_self(n1, &fresh_var),
                    bt2.subst_self(n2, &fresh_var),
                )
        }
        (
            FunConstruct {
                n: n1,
                at: at1,
                b: b1,
            },
            FunConstruct {
                n: n2,
                at: at2,
                b: b2,
            },
        ) => {
            let fresh_name = format!("%{}%{}%", n1, n2);
            let fresh_var = Var { n: &fresh_name };
            is_beta_eq(*at1, *at2)
                && is_beta_eq(b1.subst_self(n1, &fresh_var), b2.subst_self(n2, &fresh_var))
        }
        (FunDestruct { f: f1, a: a1 }, FunDestruct { f: f2, a: a2 }) => {
            is_beta_eq(*f1, *f2) && is_beta_eq(*a1, *a2)
        }
        _ => false,
    }
}

/// Beta reduce the head of the term. May panic if the type does not typecheck.
pub fn beta_reduce_head<'src>(term: Term<'src>) -> Term<'src> {
    match term {
        t @ Type => t,
        t @ Var { .. } => t,
        Let { n, v, b, .. } => beta_reduce_head(b.subst_self(n, &*v)),
        t @ FunType { .. } => t,
        t @ FunConstruct { .. } => t,
        FunDestruct { f, a } => {
            let f = beta_reduce_head(*f);
            match f {
                FunConstruct { n, b, .. } => beta_reduce_head(b.subst_self(n, &*a)),
                t @ _ => t,
            }
        }
    }
}

#[cfg(test)]
mod test_beta_reduce {
    use super::*;
    use crate::parser::term_parser;

    fn assert_correct(input: &str, val: &str) {
        let term: Term = term_parser::program(input).unwrap();
        let reduced = beta_reduce_head(term);
        assert_eq!(format!("{:?}", reduced), val);
    }

    #[test]
    fn test_beta_reduce_head() {
        assert_correct("Type", "Type");
        assert_correct("(/T:Type. T) Type", "Type");
        assert_correct("(/T:Type, x:T. x)", "/T:(Type).(/x:(T).(x))");
        assert_correct("(/T:Type, x:T. x) Type", "/x:(Type).(x)");
        assert_correct("(/T:Type, x:T. x) Type Type", "Type");
    }
}

#[cfg(test)]
mod test_beta_eq {
    use super::*;
    use crate::parser::term_parser;

    fn assert_beta_eq(t1: &str, t2: &str) {
        let t1: Term = term_parser::program(t1).unwrap();
        let t2: Term = term_parser::program(t2).unwrap();
        assert!(is_beta_eq(t1, t2));
    }

    fn assert_beta_neq(t1: &str, t2: &str) {
        let t1: Term = term_parser::program(t1).unwrap();
        let t2: Term = term_parser::program(t2).unwrap();
        assert!(!is_beta_eq(t1, t2));
    }

    #[test]
    fn test_beta_eq_simple() {
        assert_beta_eq("Type", "Type");
        assert_beta_neq("Type", "(/T:Type. T)");
    }

    #[test]
    fn test_beta_eq_alpha() {
        assert_beta_eq("(/S:Type. S)", "(/T:Type. T)");
        assert_beta_eq("(S : Type) -> Type", "(T : Type) -> Type");
        assert_beta_eq("(S : Type) -> S", "(T : Type) -> T");

        assert_beta_eq("x", "x");
        assert_beta_neq("x", "y");
        assert_beta_eq("/x:y.x", "/a:y.a");
        assert_beta_neq("/x:y.x", "/x:a.x");
        assert_beta_eq("(/x:y->y.x) (/x:y.x)", "(/a:y->a.a) (/b:y.b)");
        assert_beta_eq("(a:t)->a", "(b:t)->b");
        assert_beta_eq("(/x:Type,x:x.x) y", "(/z:y.z)")
    }

    #[test]
    fn test_beta_eq_beta() {
        assert_beta_eq("(/T:Type.T)", "(/T:Type. (/S:Type.S) T)");
        assert_beta_eq("(/T:Type.T) Type", "Type");
        assert_beta_eq("(/T:Type,e:T.e) Type Type", "Type");
        assert_beta_neq("(/T:Type.(/T:Type.T)) Type", "Type");
    }
}
