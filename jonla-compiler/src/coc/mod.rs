pub mod beta;
pub mod env;
pub mod type_check;

use crate::coc::env::Env;
use crate::coc::Expr::*;
use jonla_parser::grammar::action_result::ActionResult;
use std::fmt::{Display, Formatter};
use std::rc::Rc;

pub type W<T> = Rc<T>;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Expr {
    Type,
    Let(W<Self>, W<Self>),
    Var(usize),
    FnType(W<Self>, W<Self>),
    FnConstruct(W<Self>, W<Self>),
    FnDestruct(W<Self>, W<Self>),
}

impl Expr {
    pub fn from_action_result(value: &ActionResult, src: &str) -> Self {
        match value {
            ActionResult::Construct("Type", args) => {
                assert_eq!(args.len(), 0);
                Expr::Type
            }
            ActionResult::Construct("Let", args) => {
                assert_eq!(args.len(), 2);
                Expr::Let(
                    W::new(Expr::from_action_result(&args[0], src)),
                    W::new(Expr::from_action_result(&args[1], src)),
                )
            }
            ActionResult::Construct("Var", args) => {
                assert_eq!(args.len(), 1);
                Expr::Var(args[0].get_value(src).parse().unwrap())
            }
            ActionResult::Construct("FnType", args) => {
                assert_eq!(args.len(), 2);
                Expr::FnType(
                    W::new(Expr::from_action_result(&args[0], src)),
                    W::new(Expr::from_action_result(&args[1], src)),
                )
            }
            ActionResult::Construct("FnConstruct", args) => {
                assert_eq!(args.len(), 2);
                Expr::FnConstruct(
                    W::new(Expr::from_action_result(&args[0], src)),
                    W::new(Expr::from_action_result(&args[1], src)),
                )
            }
            ActionResult::Construct("FnDestruct", args) => {
                assert_eq!(args.len(), 2);
                Expr::FnDestruct(
                    W::new(Expr::from_action_result(&args[0], src)),
                    W::new(Expr::from_action_result(&args[1], src)),
                )
            }
            _ => unreachable!(),
        }
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Type => write!(f, "Type"),
            Let(v, b) => {
                writeln!(f, "let {v};")?;
                write!(f, "{b}")
            }
            Var(i) => write!(f, "#{i}"),
            FnType(a, b) => write!(f, "({a}) -> ({b})"),
            FnConstruct(a, b) => write!(f, "({a}). ({b})"),
            FnDestruct(a, b) => write!(f, "({a}) ({b})"),
        }
    }
}

pub type SExpr<'a> = (&'a Expr, Env<'a>);
