pub mod brh_expr;
pub mod env;
pub mod type_check;
pub mod shift_free;

use prism_parser::core::span::Span;
use prism_parser::rule_action::action_result::ActionResult;
use std::fmt::{Display, Formatter};
use prism_parser::parser::parser_instance::Arena;
use crate::coc::Expr::*;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Expr<'arn> {
    Type,
    Let(&'arn Expr<'arn>, &'arn Expr<'arn>),
    Var(usize),
    FnType(&'arn Expr<'arn>, &'arn Expr<'arn>),
    FnConstruct(&'arn Expr<'arn>, &'arn Expr<'arn>),
    FnDestruct(&'arn Expr<'arn>, &'arn Expr<'arn>),
}

#[allow(unused)]
#[derive(Clone)]
pub struct SourceInfo {
    span: Span,
}

impl<'arn> Expr<'arn> {
    pub fn from_action_result<'grm>(value: &ActionResult<'_, 'grm>, src: &'grm str, arena: &'arn Arena<Expr<'arn>>) -> &'arn Self {
        let ActionResult::Construct(span, constructor, args) = value else {
            unreachable!();
        };
        let inner = match *constructor {
            "Type" => {
                assert_eq!(args.len(), 0);
                Type
            }
            "Let" => {
                assert_eq!(args.len(), 2);
                Let(
                    Expr::from_action_result(&args[0], src, arena),
                    Expr::from_action_result(&args[1], src, arena),
                )
            }
            "Var" => {
                assert_eq!(args.len(), 1);
                Var(args[0].get_value(src).parse().unwrap())
            }
            "FnType" => {
                assert_eq!(args.len(), 2);
                FnType(
                    Expr::from_action_result(&args[0], src, arena),
                    Expr::from_action_result(&args[1], src, arena),
                )
            }
            "FnConstruct" => {
                assert_eq!(args.len(), 2);
                FnConstruct(
                    Expr::from_action_result(&args[0], src, arena),
                    Expr::from_action_result(&args[1], src, arena),
                )
            }
            "FnDestruct" => {
                assert_eq!(args.len(), 2);
                FnDestruct(
                    Expr::from_action_result(&args[0], src, arena),
                    Expr::from_action_result(&args[1], src, arena),
                )
            }
            _ => unreachable!(),
        };
        arena.alloc(inner)
    }
}

impl Display for Expr<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            Type => write!(f, "Type"),
            Let(v, b) => {
                writeln!(f, "let {v};")?;
                write!(f, "{b}")
            }
            Var(i) => write!(f, "#{i}"),
            FnType(a, b) => write!(f, "({a}) -> ({b})"),
            FnConstruct(a, b) => write!(f, "({a}) => ({b})"),
            FnDestruct(a, b) => write!(f, "({a}) ({b})"),
        }
    }
}

