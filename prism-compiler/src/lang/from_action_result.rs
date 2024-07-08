use crate::lang::{PartialExpr, TcEnv, UnionIndex, ValueOrigin};
use prism_parser::parser::parser_instance::Arena;
use prism_parser::parser::var_map::{VarMap, VarMapNode, VarMapValue};
use prism_parser::rule_action::action_result::ActionResult;
use rpds::{List, RedBlackTreeMap};

#[derive(Clone)]
struct Scope<'arn, 'grm> {
    vars: VarMap<'arn, 'grm>,
    depth: usize,
}

impl<'arn, 'grm> Default for Scope<'arn, 'grm> {
    fn default() -> Self {
        Scope {
            vars: VarMap::default(),
            depth: 0,
        }
    }
}

impl<'arn, 'grm> Scope<'arn, 'grm> {
    pub fn insert_name(&self, key: &'arn str, arena: &'arn Arena<VarMapNode<'arn, 'grm>>) -> Self {
        Self {
            vars: self
                .vars
                .insert(key, VarMapValue::ByIndex(self.depth), arena),
            depth: self.depth + 1,
        }
    }

    pub fn get(&self, key: &'arn str) -> &VarMapValue {
        self.vars.get(key).expect("Name exists")
    }
}

#[derive(Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct Guid(pub usize);

impl TcEnv {
    pub fn insert_from_action_result<'arn, 'grm>(
        &mut self,
        value: &ActionResult<'arn, 'grm>,
        program: &'arn str,
        arena: &'arn Arena<VarMapNode<'arn, 'grm>>,
    ) -> UnionIndex {
        self.insert_from_action_result_rec(value, program, &Scope::default(), arena)
    }

    fn insert_from_action_result_rec<'arn, 'grm>(
        &mut self,
        value: &ActionResult<'arn, 'grm>,
        program: &'arn str,
        vars: &Scope<'arn, 'grm>,
        arena: &'arn Arena<VarMapNode<'arn, 'grm>>,
    ) -> UnionIndex {
        let (inner, inner_span) = match value {
            ActionResult::Construct(span, constructor, args) => (
                match *constructor {
                    "Type" => {
                        assert_eq!(args.len(), 0);
                        PartialExpr::Type
                    }
                    "Let" => {
                        assert_eq!(args.len(), 3);
                        let name = Self::parse_name(&args[0], program);

                        let v = self.insert_from_action_result_rec(&args[1], program, vars, arena);
                        let b = self.insert_from_action_result_rec(
                            &args[2],
                            program,
                            &vars.insert_name(name, arena),
                            arena,
                        );

                        PartialExpr::Let(v, b)
                    }
                    _ => unreachable!(),
                },
                *span,
            ),
            ActionResult::Value(span) => {
                let name = Self::parse_name(value, program);

                let e = if name == "_" {
                    PartialExpr::Free
                } else {
                    match vars.get(name) {
                        VarMapValue::Expr(_) => unreachable!(),
                        VarMapValue::Value(ar) => todo!(),
                        VarMapValue::ByIndex(ix) => PartialExpr::DeBruijnIndex(vars.depth - ix - 1),
                    }
                };
                (e, *span)
            }
            _ => unreachable!(),
        };
        self.store(inner, ValueOrigin::SourceCode(inner_span))
    }

    fn parse_name<'arn, 'grm>(ar: &ActionResult<'arn, 'grm>, program: &'arn str) -> &'arn str {
        let ActionResult::Value(span) = ar else {
            unreachable!()
        };
        &program[*span]
    }

    fn parse_guid(ar: &ActionResult) -> Guid {
        let ActionResult::Guid(v) = ar else {
            unreachable!()
        };
        Guid(*v)
    }
}
