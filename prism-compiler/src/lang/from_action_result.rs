use crate::lang::error::TypeError;
use crate::lang::{PartialExpr, TcEnv, UnionIndex, ValueOrigin};
use prism_parser::parser::parser_instance::Arena;
use prism_parser::parser::var_map::{VarMap, VarMapNode, VarMapValue};
use prism_parser::rule_action::action_result::ActionResult;
use rpds::RedBlackTreeMap;
use std::borrow::Cow;

#[derive(Clone, Debug)]
enum ScopeValue<'arn, 'grm> {
    FromEnv(usize),
    FromGrammar(
        prism_parser::core::cow::Cow<'arn, ActionResult<'arn, 'grm>>,
        Scope<'arn, 'grm>,
    ),
}

#[derive(Clone, Debug)]
struct Scope<'arn, 'grm> {
    names: RedBlackTreeMap<&'arn str, ScopeValue<'arn, 'grm>>,
    named_scopes: RedBlackTreeMap<Guid, RedBlackTreeMap<&'arn str, ScopeValue<'arn, 'grm>>>,
    depth: usize,
    hygienic_decls: RedBlackTreeMap<&'arn str, usize>,
}

impl<'arn, 'grm> Default for Scope<'arn, 'grm> {
    fn default() -> Self {
        Scope {
            names: Default::default(),
            named_scopes: RedBlackTreeMap::default(),
            depth: 0,
            hygienic_decls: Default::default(),
        }
    }
}

impl<'arn, 'grm> Scope<'arn, 'grm> {
    pub fn insert_name(&self, key: &'arn str, program: &'arn str) -> Self {
        Self {
            depth: self.depth + 1,
            ..self.clone()
        }.insert_name_at(key, self.depth, program)
    }

    pub fn insert_name_at(&self, key: &'arn str, depth: usize, program: &'arn str) -> Self {
        let names = self.names.insert(key, ScopeValue::FromEnv(depth));
        let hygienic_decls = if let Some(ScopeValue::FromGrammar(ar, _)) = self.names.get(key) {
            let new_name = TcEnv::parse_name(ar, program);
            self.hygienic_decls.insert(new_name, depth)
        } else {
            self.hygienic_decls.clone()
        };

        Self {
            names,
            hygienic_decls,
            ..self.clone()
        }
    }

    pub fn get(&self, key: &str) -> Option<&ScopeValue<'arn, 'grm>> {
        self.names.get(key)
    }

    pub fn extend_with_ars(&self, new_vars: &VarMap<'arn, 'grm>, vars: &Self) -> Self {
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
            ..self.clone()
        }
    }

    pub fn insert_jump(&self, guid: Guid) -> Self {
        Scope {
            names: self.names.clone(),
            named_scopes: self.named_scopes.insert(guid, self.names.clone()),
            depth: self.depth,
            hygienic_decls: self.hygienic_decls.clone()
        }
    }

    pub fn jump(&self, guid: Guid) -> Self {
        Scope {
            names: self.named_scopes[&guid].clone(),
            named_scopes: self.named_scopes.clone(),
            depth: self.depth,
            hygienic_decls: self.hygienic_decls.clone()
        }
    }
}

#[derive(Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Debug)]
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
                        assert_eq!(args.len(), 2);
                        let guid = Self::parse_guid(&args[1]);
                        return self.insert_from_action_result_rec(
                            &args[0],
                            program,
                            &vars.insert_jump(guid),
                        );
                    }
                    "TypeAssert" => {
                        assert_eq!(args.len(), 2);
                        
                        let e = self.insert_from_action_result_rec(&args[0], program, vars);
                        let typ = self.insert_from_action_result_rec(&args[1], program, vars);
                        
                        PartialExpr::TypeAssert(e, typ)
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
                            // Create a new scope based on the current depth and `scope_vars`
                            let mut scope_vars_with_hygienic_decls = Scope {
                                depth: vars.depth,
                                ..scope_vars.clone()
                            };

                            // Insert hygienically declared variables into the scope
                            for (k, v) in &vars.hygienic_decls {
                                scope_vars_with_hygienic_decls = scope_vars_with_hygienic_decls.insert_name_at(k, *v, program);
                            }

                            // Parse the value in the new scope
                            return self.insert_from_action_result_rec(
                                ar,
                                program,
                                &scope_vars_with_hygienic_decls,
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
                let vars = vars.jump(guid).extend_with_ars(new_vars, vars);

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
