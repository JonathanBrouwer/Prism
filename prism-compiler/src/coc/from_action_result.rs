use crate::coc::{PartialExpr, TcEnv};
use crate::union_find::{UnionFind, UnionIndex};
use prism_parser::rule_action::action_result::ActionResult;

impl TcEnv {
    pub fn from_action_result<'grm>(value: &ActionResult<'_, 'grm>, src: &'grm str) -> Self {
        let mut s = Self {
            uf: UnionFind::new(),
            uf_values: Vec::default(),
            errors: Vec::new(),
            root: UnionIndex(0),
        };
        let type_type = s.insert_union_index(PartialExpr::Type);
        debug_assert_eq!(type_type.0, 0);
        s.root = s.insert_from_action_result(value, src);
        s
    }

    pub fn insert_from_action_result<'grm>(
        &mut self,
        value: &ActionResult<'_, 'grm>,
        src: &'grm str,
    ) -> UnionIndex {
        let ActionResult::Construct(_span, constructor, args) = value else {
            unreachable!();
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
            "Var" => {
                assert_eq!(args.len(), 1);
                PartialExpr::Var(args[0].get_value(src).parse().unwrap())
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
            _ => unreachable!(),
        };
        self.insert_union_index(inner)
    }
}
