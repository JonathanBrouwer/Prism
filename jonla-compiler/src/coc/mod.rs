pub mod beta;
pub mod env;
pub mod type_check;

// use crate::coc::env::Env;
use crate::coc::ExprInner::*;
use jonla_parser::grammar::action_result::ActionResult;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
use jonla_parser::core::span::Span;

pub type W<T> = Rc<T>;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Expr<M: Clone>(M, ExprInner<M>);

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum ExprInner<M: Clone> {
    Type,
    Let(W<Expr<M>>, W<Expr<M>>),
    Var(usize),
    FnType(W<Expr<M>>, W<Expr<M>>),
    FnConstruct(W<Expr<M>>, W<Expr<M>>),
    FnDestruct(W<Expr<M>>, W<Expr<M>>),
}

pub struct SourceInfo {
    span: Span,
}

impl Expr<()> {
    pub fn from_action_result(value: &ActionResult, src: &str) -> Self {
        Expr((), match value {
            ActionResult::Construct(_, "Type", args) => {
                assert_eq!(args.len(), 0);
                Type
            }
            ActionResult::Construct(_, "Let", args) => {
                assert_eq!(args.len(), 2);
                Let(
                    W::new(Expr::from_action_result(&args[0], src)),
                    W::new(Expr::from_action_result(&args[1], src)),
                )
            }
            ActionResult::Construct(_, "Var", args) => {
                assert_eq!(args.len(), 1);
                Var(args[0].get_value(src).parse().unwrap())
            }
            ActionResult::Construct(_, "FnType", args) => {
                assert_eq!(args.len(), 2);
                FnType(
                    W::new(Expr::from_action_result(&args[0], src)),
                    W::new(Expr::from_action_result(&args[1], src)),
                )
            }
            ActionResult::Construct(_, "FnConstruct", args) => {
                assert_eq!(args.len(), 2);
                FnConstruct(
                    W::new(Expr::from_action_result(&args[0], src)),
                    W::new(Expr::from_action_result(&args[1], src)),
                )
            }
            ActionResult::Construct(_, "FnDestruct", args) => {
                assert_eq!(args.len(), 2);
                FnDestruct(
                    W::new(Expr::from_action_result(&args[0], src)),
                    W::new(Expr::from_action_result(&args[1], src)),
                )
            }
            _ => unreachable!(),
        })
    }
}

impl<M: Clone> Display for Expr<M> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.1 {
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

// pub type SExpr<'a, M> = (&'a Expr<M>, Env<'a, M>);
