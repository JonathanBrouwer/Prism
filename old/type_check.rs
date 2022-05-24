use crate::beta_eq::{beta_reduce_head, is_beta_eq};
use crate::term::Term;
use crate::term::Term::*;
use crate::type_check::TypeCheckError::*;
use std::collections::HashMap;

#[derive(Debug)]
pub enum TypeCheckError<'src> {
    NameNotFound(&'src str),
    ExpectEqual(Term<'src>, Term<'src>),
    ExpectFunction(Term<'src>),
}

pub fn type_check<'src>(
    term: &Term<'src>,
    vars: &mut HashMap<&'src str, Term<'src>>,
) -> Result<Term<'src>, TypeCheckError<'src>> {
    match term {
        Type {} => Ok(Type),
        Var { n } => vars.get(n).cloned().ok_or(NameNotFound(*n)),
        Let { n, t, v, b } => {
            let tt = type_check(t.as_ref(), vars)?;
            expect_beta_eq(tt, Type)?;

            let vt = type_check(v, vars)?;
            expect_beta_eq(vt, t.as_ref().clone())?;

            //We don't need to add n to vars, as it is substituted (which is a difference between let and fun)
            type_check(&b.as_ref().clone().subst_self(n, v), vars)
        }
        FunType { n, at, bt } => {
            let att = type_check(at.as_ref(), vars)?;
            expect_beta_eq(att, Type)?;

            let n_oldv = vars.insert(n, at.as_ref().clone());
            let btt = type_check(bt.as_ref(), vars)?;
            expect_beta_eq(btt, Type)?;
            if let Some(n_oldv) = n_oldv {
                vars.insert(n, n_oldv);
            } else {
                vars.remove(n);
            }

            Ok(Type)
        }
        FunConstruct { n, at, b } => {
            let att = type_check(at.as_ref(), vars)?;
            expect_beta_eq(att, Type)?;

            let n_oldv = vars.insert(n, at.as_ref().clone());
            let bt = type_check(b.as_ref(), vars)?;
            if let Some(n_oldv) = n_oldv {
                vars.insert(n, n_oldv);
            } else {
                vars.remove(n);
            }

            Ok(FunType {
                n: *n,
                at: (*at).clone(),
                bt: box bt,
            })
        }
        FunDestruct { f, a } => {
            let ft = beta_reduce_head(type_check(f, vars)?);
            let at_given = type_check(a.as_ref(), vars)?;

            match ft {
                FunType {
                    n,
                    at: at_declared,
                    bt,
                } => {
                    expect_beta_eq(at_given, *at_declared)?;
                    Ok(bt.subst_self(n, a))
                }
                _ => Err(ExpectFunction(f.as_ref().clone())),
            }
        }
    }
}

fn expect_beta_eq<'src>(t1: Term<'src>, t2: Term<'src>) -> Result<(), TypeCheckError<'src>> {
    if is_beta_eq(t1.clone(), t2.clone()) {
        Ok(())
    } else {
        Err(ExpectEqual(t1, t2))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::term_parser;

    fn assert_type(term: &str, typ: &str) {
        let term: Term = term_parser::program(term).unwrap();
        let typed = type_check(&term, &mut HashMap::new()).unwrap();
        assert_eq!(format!("{:?}", typed), typ);
    }

    #[test]
    fn simple() {
        assert_type("Type", "Type");
        assert_type("/x:Type.(t:Type)->x", "(x:Type)->(Type)");
        assert_type("(/x:Type.((t:x)->x)) Type", "Type");
        assert_type("(/x:Type.x) Type", "Type");
    }

    #[test]
    fn identities() {
        assert_type("(/T:Type,x:T.x)", "(T:Type)->((x:T)->(T))");
        assert_type("(/T:Type,x:T.x) Type", "(x:Type)->(Type)");
    }

    #[test]
    fn complex_subst() {
        assert_type(
            "let x : (T : Type) -> T -> T = / T : Type, x : T. x;x (Type -> Type)",
            "(x:(_:Type)->(Type))->((_:Type)->(Type))",
        );
        assert_type(
            "let x : (T : Type) -> T -> T = / T : Type, x : T. x;x ((T: Type) -> T)",
            "(x:(T:Type)->(T))->((T:Type)->(T))",
        );
    }

    #[test]
    fn id_with_id() {
        assert_type(
            "/a:Type. (/x: (/y: Type. y) a. x)",
            "(a:Type)->((x:(/y:(Type).(y) a))->((/y:(Type).(y) a)))",
        );
        assert_type(
            "(/a:Type. (/x: (/y: Type. y) a. x)) Type",
            "(x:(/y:(Type).(y) Type))->((/y:(Type).(y) Type))", // (x:Type)->(Type)
        );
        assert_type(
            "(/a:Type. (/x: (/y: Type. y) a. x)) Type (Type -> Type)",
            "(/y:(Type).(y) Type)", // Type
        );
    }

    #[test]
    fn church_wrapper() {
        assert_type(
            include_str!("../resources/church_wrapper.jl"),
            "(p:Type)->((a:(/p:(Type).((c:Type)->((_:(_:p)->(c))->(c))) p))->(p))",
        );
    }

    #[test]
    fn church_and() {
        assert_type(
            &format!(
                "{}{}",
                include_str!("../resources/church_and.jl"),
                "conj Type Type (Type -> Type) (Type -> Type -> Type)"
            ),
            "(c:Type)->((f:(_:Type)->((_:Type)->(c)))->(c))",
        );
        assert_type(
            &format!(
                "{}{}",
                include_str!("../resources/church_and.jl"),
                "proj1 Type Type (conj Type Type (Type -> Type) (Type -> Type -> Type))"
            ),
            "Type",
        );
        assert_type(
            &format!(
                "{}{}",
                include_str!("../resources/church_and.jl"),
                "proj2 Type Type (conj Type Type (Type -> Type) (Type -> Type -> Type))"
            ),
            "Type",
        );
    }
}
