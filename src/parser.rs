use crate::term::Term;

peg::parser! {
    pub grammar term_parser() for str {
        rule w() = [' ']
        rule _ = w()*
        rule nls() = ['\n' | ';' | '\r']*

        rule identifier() -> &'input str
            = x: quiet!{$([ 'a'..='z' | 'A'..='Z' | '_' ]['a'..='z' | 'A'..='Z' | '0'..='9' | '_' ]*)} / expected!("identifier")

        pub rule program() -> Term<'input> = _ t:term() _ {t}

        rule term() -> Term<'input> = precedence!{
            "let" _ n:identifier() _ ":" _ t:term() _ "=" _ v:term() _ nls() _ b:term() { Term::Let{n, t:box t, v:box v, b:box b} }
            "/" _ subs:((x:identifier() _ ":" _ t:term() { (x, t) })++(_ "," _)) _ "." _ b:term() { subs.into_iter().rev().fold(b, |b, n| Term::FunConstruct{n:n.0, at:box n.1, b:box b}) }
            "(" _ n:identifier() _ ":" _ at:term() _ ")" _ "->" _ bt:term() { Term::FunType{n, at: box at, bt: box bt} }
            at:subterm() _ "->" _ bt:term() { Term::FunType{n:"_", at: box at, bt: box bt} }
            sub:subterm() { sub }
        }

        rule subterm() -> Term<'input> = precedence!{
            subs:(t:subsubterm())**<2,>(w()+) { subs.into_iter().reduce(|f,a| Term::FunDestruct{f: box f, a: box a}).unwrap() }
            sub:subsubterm() { sub }
        }

        rule subsubterm() -> Term<'input> = precedence!{
            "Type" !(['a'..='z' | 'A'..='Z']) { Term::Type }
            n:identifier() { Term::Var{n} }
            "(" _ t:term() _ ")" { t }
        }

    }
}

#[cfg(test)]
mod parser_tests {
    use super::*;

    #[test]
    fn test_type() {
        assert_eq!(
            &format!("{:?}", term_parser::program("Type").unwrap()),
            "Type"
        );
    }

    #[test]
    fn test_fun_constr() {
        assert_eq!(
            &format!("{:?}", term_parser::program("/T:Type.T").unwrap()),
            "/T:(Type).(T)"
        );
        assert_eq!(
            &format!("{:?}", term_parser::program("/ T : Type . T").unwrap()),
            "/T:(Type).(T)"
        );
        assert_eq!(
            &format!("{:?}", term_parser::program("/T:Type./e:T.e").unwrap()),
            "/T:(Type).(/e:(T).(e))"
        );
        assert_eq!(
            &format!("{:?}", term_parser::program("/T:Type, e:T. e").unwrap()),
            "/T:(Type).(/e:(T).(e))"
        );
        assert_eq!(
            &format!(
                "{:?}",
                term_parser::program("/ T : Type , e : T . e").unwrap()
            ),
            "/T:(Type).(/e:(T).(e))"
        );
    }

    #[test]
    fn test_fun_destr() {
        assert_eq!(
            &format!(
                "{:?}",
                term_parser::program("(/T:Type, e:T. e) Type Type").unwrap()
            ),
            "((/T:(Type).(/e:(T).(e)) Type) Type)"
        );
        assert_eq!(
            &format!(
                "{:?}",
                term_parser::program("( / T : Type , e : T . e ) Type Type").unwrap()
            ),
            "((/T:(Type).(/e:(T).(e)) Type) Type)"
        );
    }

    #[test]
    fn test_fun_type() {
        assert_eq!(
            &format!(
                "{:?}",
                term_parser::program("(/T:Type, e:(x: T)->T. e) Type (/x:Type.x)").unwrap()
            ),
            "((/T:(Type).(/e:((x:T)->(T)).(e)) Type) /x:(Type).(x))"
        );
        assert_eq!(
            &format!(
                "{:?}",
                term_parser::program("/f : (x: Type) -> (y : Type) -> Type . Type").unwrap()
            ),
            "/f:((x:Type)->((y:Type)->(Type))).(Type)"
        );
        assert_eq!(
            &format!(
                "{:?}",
                term_parser::program("/f : Type -> Type -> Type . Type").unwrap()
            ),
            "/f:((_:Type)->((_:Type)->(Type))).(Type)"
        );
    }
}
