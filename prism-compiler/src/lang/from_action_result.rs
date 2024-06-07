use crate::lang::UnionIndex;
use crate::lang::{PartialExpr, TcEnv};
use prism_parser::rule_action::action_result::ActionResult;

impl TcEnv {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_from_action_result<'grm>(
        &mut self,
        value: &ActionResult<'_, 'grm>,
        src: &'grm str,
    ) -> UnionIndex {
        let ActionResult::Construct(span, constructor, args) = value else {
            unreachable!("Parsing an expression always returns a Construct");
        };
        let inner = match *constructor {
            "Type" => {
                assert_eq!(args.len(), 0);
                PartialExpr::Type
            }
            "Let" => {
                assert_eq!(args.len(), 2);
                PartialExpr::Let(
                    self.insert_from_action_result(&args[0], src),
                    self.insert_from_action_result(&args[1], src),
                )
            }
            "DeBruijnIndex" => {
                assert_eq!(args.len(), 1);
                PartialExpr::DeBruijnIndex(args[0].get_value(src).parse().unwrap())
            }
            "FnType" => {
                assert_eq!(args.len(), 2);
                PartialExpr::FnType(
                    self.insert_from_action_result(&args[0], src),
                    self.insert_from_action_result(&args[1], src),
                )
            }
            "FnConstruct" => {
                assert_eq!(args.len(), 2);
                PartialExpr::FnConstruct(
                    self.insert_from_action_result(&args[0], src),
                    self.insert_from_action_result(&args[1], src),
                )
            }
            "FnDestruct" => {
                assert_eq!(args.len(), 2);
                PartialExpr::FnDestruct(
                    self.insert_from_action_result(&args[0], src),
                    self.insert_from_action_result(&args[1], src),
                )
            }
            "Free" => {
                assert_eq!(args.len(), 0);
                PartialExpr::Free
            }
            _ => unreachable!(),
        };
        self.store_from_source(inner, *span)
    }
}
