use crate::lang::error::TypeError;
use crate::lang::{PartialExpr, TcEnv, UnionIndex, ValueOrigin};
use prism_parser::parser::parser_instance::Arena;
use prism_parser::parser::var_map::{VarMap, VarMapNode, VarMapValue};
use prism_parser::rule_action::action_result::ActionResult;
use rpds::RedBlackTreeMap;
use std::borrow::Cow;

enum ScopeValue<'arn, 'grm> {
    FromEnv(usize),
    FromGrammar(
        prism_parser::core::cow::Cow<'arn, ActionResult<'arn, 'grm>>,
        Scope<'arn, 'grm>,
    ),
}

#[derive(Clone)]
struct Scope<'arn, 'grm> {
    names: RedBlackTreeMap<&'arn str, ScopeValue<'arn, 'grm>>,
    named_scopes: RedBlackTreeMap<Guid, RedBlackTreeMap<&'arn str, ScopeValue<'arn, 'grm>>>,
    depth: usize,
    hygienic_declarations: RedBlackTreeMap<&'arn str, ScopeValue<'arn, 'grm>>,
}

impl<'arn, 'grm> Default for Scope<'arn, 'grm> {
    fn default() -> Self {
        Scope {
            names: Default::default(),
            named_scopes: RedBlackTreeMap::default(),
            depth: 0,
            hygienic_declarations: Default::default(),
        }
    }
}

impl<'arn, 'grm> Scope<'arn, 'grm> {
    pub fn insert_name(&self, key: &'arn str, program: &'arn str) -> Self {
        let names = self.names.insert(key, ScopeValue::FromEnv(self.depth));
        let hygienic_declarations = if let Some(ScopeValue::FromGrammar(ar, ar_scope)) = self.names.get(key) {
            //TODO use ar_scope
            let new_name = TcEnv::parse_name(ar, program);
            self.hygienic_declarations.insert(new_name, ScopeValue::FromEnv(self.depth))
        } else {
            self.hygienic_declarations.clone()
        };

        Self {
            names,
            named_scopes: self.named_scopes.clone(),
            depth: self.depth + 1,
            hygienic_declarations,
        }
    }

    pub fn get(&self, key: &str) -> Option<&ScopeValue<'arn, 'grm>> {
        self.names.get(key)
    }

    pub fn extend_without_depth(&self, new_vars: &VarMap<'arn, 'grm>, vars: &Self) -> Self {
        let mut names = self.names.clone();
        for (name, value) in new_vars.iter_cloned() {
            match value {
                VarMapValue::Expr(_) => continue,
                VarMapValue::Value(ar) => {
                    names.insert_mut(name, ScopeValue::FromGrammar(ar.clone(), vars.clone()));
                }
            }
        }

        Self {
            names,
            named_scopes: self.named_scopes.clone(),
            depth: self.depth,
            hygienic_declarations: self.hygienic_declarations.clone()
        }
    }

    pub fn insert_jump(&self, guid: Guid) -> Self {
        Scope {
            names: self.names.clone(),
            named_scopes: self.named_scopes.insert(guid, self.names.clone()),
            depth: self.depth,
            hygienic_declarations: self.hygienic_declarations.clone()
        }
    }

    pub fn jump(&self, guid: Guid) -> Self {
        Scope {
            names: self.named_scopes[&guid].clone(),
            named_scopes: self.named_scopes.clone(),
            depth: self.depth,
            hygienic_declarations: self.hygienic_declarations.clone()
        }
    }

    pub fn with_depth(&self, depth_from: &Self) -> Self {
        Self {
            names: self.names.clone(),
            named_scopes: self.named_scopes.clone(),
            depth: depth_from.depth,
            hygienic_declarations: self.hygienic_declarations.clone()
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
        _arena: &'arn Arena<VarMapNode<'arn, 'grm>>,
    ) -> UnionIndex {
        self.insert_from_action_result_rec(value, program, &Scope::default())
    }

    fn insert_from_action_result_rec<'arn, 'grm>(
        &mut self,
        value: &ActionResult<'arn, 'grm>,
        program: &'arn str,
        vars: &Scope<'arn, 'grm>,
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

                        let v = self.insert_from_action_result_rec(&args[1], program, vars);
                        let b = self.insert_from_action_result_rec(
                            &args[2],
                            program,
                            &vars.insert_name(name, program),
                        );

                        PartialExpr::Let(v, b)
                    }
                    "FnType" => {
                        assert_eq!(args.len(), 3);
                        let name = Self::parse_name(&args[0], program);

                        let v = self.insert_from_action_result_rec(&args[1], program, vars);
                        let b = self.insert_from_action_result_rec(
                            &args[2],
                            program,
                            &vars.insert_name(name, program),
                        );

                        PartialExpr::FnType(v, b)
                    }
                    "FnConstruct" => {
                        assert_eq!(args.len(), 3);
                        let name = Self::parse_name(&args[0], program);

                        let v = self.insert_from_action_result_rec(&args[1], program, vars);
                        let b = self.insert_from_action_result_rec(
                            &args[2],
                            program,
                            &vars.insert_name(name, program),
                        );

                        PartialExpr::FnConstruct(v, b)
                    }
                    "FnDestruct" => {
                        assert_eq!(args.len(), 2);

                        let f = self.insert_from_action_result_rec(&args[0], program, vars);
                        let v = self.insert_from_action_result_rec(&args[1], program, vars);

                        PartialExpr::FnDestruct(f, v)
                    }
                    "ScopeDefine" => {
                        let guid = Self::parse_guid(&args[1]);
                        return self.insert_from_action_result_rec(
                            &args[0],
                            program,
                            &vars.insert_jump(guid),
                        );
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
                        }
                        Some(ScopeValue::FromGrammar(ar, scope_vars)) => {
                            return self.insert_from_action_result_rec(
                                ar,
                                program,
                                &scope_vars.with_depth(vars),
                            )
                        }
                        Some(ScopeValue::FromEnv(ix)) => {
                            PartialExpr::DeBruijnIndex(vars.depth - ix - 1)
                        }
                    }
                };
                (e, *span)
            }
            ActionResult::WithEnv(new_vars, ar) => {
                let ActionResult::Construct(_span, "ScopeEnter", args) = ar else {
                    unreachable!()
                };
                let guid = Self::parse_guid(&args[1]);
                let vars = vars.jump(guid).extend_without_depth(new_vars, vars);

                return self.insert_from_action_result_rec(&args[0], program, &vars);
            }
            _ => unreachable!(),
        };
        self.store(inner, ValueOrigin::SourceCode(inner_span))
    }

    fn parse_name<'arn, 'grm>(ar: &ActionResult<'arn, 'grm>, program: &'arn str) -> &'arn str {
        match ar {
            ActionResult::Value(span) => &program[*span],
            ActionResult::Literal(l) => match l.to_cow() {
                Cow::Borrowed(s) => s,
                Cow::Owned(_) => unreachable!(),
            },
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
