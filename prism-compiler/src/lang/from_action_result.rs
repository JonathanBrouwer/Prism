use std::borrow::Cow;
use crate::lang::{PartialExpr, TcEnv, UnionIndex, ValueOrigin};
use prism_parser::parser::parser_instance::Arena;
use prism_parser::parser::var_map::{VarMap, VarMapNode, VarMapValue};
use prism_parser::rule_action::action_result::ActionResult;
use rpds::{List, RedBlackTreeMap};
use crate::lang::error::TypeError;

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

    pub fn get(&self, key: &'arn str) -> Option<&VarMapValue<'arn, 'grm>> {
        self.vars.get(key)
    }

    pub fn extend_without_depth(&self, new_vars: &VarMap<'arn, 'grm>, arena: &'arn Arena<VarMapNode<'arn, 'grm>>) -> Self {
        Self {
            vars: self.vars.extend(new_vars.iter_cloned(), arena),
            depth: self.depth,
        }
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
                    "FnType" => {
                        assert_eq!(args.len(), 3);
                        let name = Self::parse_name(&args[0], program);

                        let v = self.insert_from_action_result_rec(&args[1], program, vars, arena);
                        let b = self.insert_from_action_result_rec(
                            &args[2],
                            program,
                            &vars.insert_name(name, arena),
                            arena,
                        );

                        PartialExpr::FnType(v, b)
                    }
                    "FnConstruct" => {
                        assert_eq!(args.len(), 3);
                        let name = Self::parse_name(&args[0], program);

                        let v = self.insert_from_action_result_rec(&args[1], program, vars, arena);
                        let b = self.insert_from_action_result_rec(
                            &args[2],
                            program,
                            &vars.insert_name(name, arena),
                            arena,
                        );

                        PartialExpr::FnConstruct(v, b)
                    }
                    "FnDestruct" => {
                        assert_eq!(args.len(), 2);

                        let f = self.insert_from_action_result_rec(&args[0], program, vars, arena);
                        let v = self.insert_from_action_result_rec(&args[1], program, vars, arena);

                        PartialExpr::FnDestruct(f, v)
                    }
                    "ScopeDefine" => {
                        //TODO
                        let guid = Self::parse_guid(&args[1]);
                        let b = self.insert_from_action_result_rec(&args[0], program, vars, arena);
                        return b;
                    },
                    "ScopeEnter" => {
                        //TODO
                        let guid = Self::parse_guid(&args[1]);
                        let b = self.insert_from_action_result_rec(&args[0], program, vars, arena);
                        return b;
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
                        None => {
                            self.errors.push(TypeError::UnknownName(*span));
                            PartialExpr::Free
                        },
                        Some(VarMapValue::Expr(_)) => unreachable!(),
                        Some(VarMapValue::Value(ar)) => {
                            return self.insert_from_action_result_rec(
                                ar,
                                program,
                                vars, arena
                            )
                        },
                        Some(VarMapValue::ByIndex(ix)) => PartialExpr::DeBruijnIndex(vars.depth - ix - 1),
                    }
                };
                (e, *span)
            }
            ActionResult::WithEnv(new_vars, ar) => {
                return self.insert_from_action_result_rec(ar, program, &vars.extend_without_depth(new_vars, arena), arena);
            }
            _ => unreachable!(),
        };
        self.store(inner, ValueOrigin::SourceCode(inner_span))
    }

    fn parse_name<'arn, 'grm>(ar: &ActionResult<'arn, 'grm>, program: &'arn str) -> &'arn str {
        match ar {
            ActionResult::Value(span) => &program[*span],
            ActionResult::Literal(l) => {
                match l.to_cow() {
                    Cow::Borrowed(s) => s,
                    Cow::Owned(_) => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }

    fn parse_guid(ar: &ActionResult) -> Guid {
        let ActionResult::Guid(v) = ar else {
            unreachable!()
        };
        Guid(*v)
    }
}
