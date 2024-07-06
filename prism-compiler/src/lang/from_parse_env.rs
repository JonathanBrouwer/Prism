use rpds::{List, RedBlackTreeMap};
use crate::desugar::{Guid, ParseEnv, ParseIndex, SourceExpr};
use crate::lang::error::TypeError;
use crate::lang::{PartialExpr, TcEnv, UnionIndex, ValueOrigin};

/// Stores the variables in scope + the depth of the scope
#[derive(Clone)]
struct Scope<'a> {
    names_stack: List<RedBlackTreeMap<&'a str, usize>>,
    named_scopes: RedBlackTreeMap<Guid, RedBlackTreeMap<&'a str, usize>>,
    depth: usize,
}

impl<'a> Default for Scope<'a> {
    fn default() -> Self {
        Scope {
            names_stack: List::new().push_front(RedBlackTreeMap::default()),
            named_scopes: RedBlackTreeMap::default(),
            depth: 0,
        }
    }
}

impl<'a> Scope<'a> {
    pub fn insert_name(&self, key: &'a str) -> Self {
        let names = self.names_stack.first().expect("Scopes not empty").insert(key, self.depth);
        let names_stack = self.names_stack.drop_first().expect("Scopes not empty").push_front(names);

        Scope {
            names_stack,
            named_scopes: self.named_scopes.clone(),
            depth: self.depth + 1,
        }
    }

    pub fn insert_jump(&self, guid: Guid) -> Self {
        Scope {
            names_stack: self.names_stack.clone(),
            named_scopes: self.named_scopes.insert(guid, self.names_stack.first().expect("Scopes not empty").clone()),
            depth: self.depth,
        }
    }
    
    pub fn jump(&self, guid: Guid) -> Self {
        let names_stack = self.names_stack.push_front(self.named_scopes[&guid].clone());

        Scope {
            names_stack,
            named_scopes: self.named_scopes.clone(),
            depth: self.depth,
        }
    }

    pub fn unjump(&self) -> Self {
        Scope {
            names_stack: self.names_stack.drop_first().expect("Scopes not empty"),
            named_scopes: self.named_scopes.clone(),
            depth: self.depth,
        }
    }
}

impl TcEnv {
    pub fn insert_parse_env(&mut self, parse_env: &ParseEnv, root: ParseIndex) -> UnionIndex {
        let mut scopes = vec![Scope::default(); parse_env.values().len()];

        let start = self.values.len();
        let take_count = root.index() + 1;

        self.values.resize(start + take_count, PartialExpr::Free);
        self.value_origins
            .resize(start + take_count, ValueOrigin::Failure);

        for (i, (expr, span)) in parse_env
            .values()
            .iter()
            .zip(parse_env.value_spans().iter())
            .take(take_count)
            .enumerate()
            .rev()
        {
            self.value_origins[start + i] = ValueOrigin::SourceCode(*span);
            self.values[start + i] = match expr {
                SourceExpr::Type => PartialExpr::Type,
                SourceExpr::Let(name, value, body) => {
                    scopes[value.index()] = scopes[i].clone();
                    scopes[body.index()] = scopes[i].insert_name(name);
                    PartialExpr::Let(
                        UnionIndex(value.index() + start),
                        UnionIndex(body.index() + start),
                    )
                }
                SourceExpr::Variable(name) => {
                    if name == "_" {
                        PartialExpr::Free
                    } else if let Some(depth) = scopes[i].names_stack.first().expect("Scopes not empty").get(name.as_str()) {
                        PartialExpr::DeBruijnIndex(scopes[i].depth - depth - 1)
                    } else {
                        self.errors.push(TypeError::UnknownName(*span));
                        PartialExpr::Free
                    }
                }
                SourceExpr::FnType(name, arg_type, return_type) => {
                    scopes[arg_type.index()] = scopes[i].clone();
                    scopes[return_type.index()] = scopes[i].insert_name(name);
                    PartialExpr::FnType(
                        UnionIndex(arg_type.index() + start),
                        UnionIndex(return_type.index() + start),
                    )
                }
                SourceExpr::FnConstruct(name, arg_type, body) => {
                    scopes[arg_type.index()] = scopes[i].clone();
                    scopes[body.index()] = scopes[i].insert_name(name);
                    PartialExpr::FnConstruct(
                        UnionIndex(arg_type.index() + start),
                        UnionIndex(body.index() + start),
                    )
                }
                SourceExpr::FnDestruct(function, arg) => {
                    scopes[function.index()] = scopes[i].clone();
                    scopes[arg.index()] = scopes[i].clone();
                    PartialExpr::FnDestruct(
                        UnionIndex(function.index() + start),
                        UnionIndex(arg.index() + start),
                    )
                }
                SourceExpr::ScopeDefine(v, guid) => {
                    scopes[v.index()] = scopes[i].insert_jump(*guid);
                    PartialExpr::Shift(UnionIndex(v.index() + start), 0)
                }
                SourceExpr::ScopeEnter(v, guid) => {
                    scopes[v.index()] = scopes[i].jump(*guid);
                    PartialExpr::Shift(UnionIndex(v.index() + start), 0)
                }
                SourceExpr::ScopeExit(v) => {
                    scopes[v.index()] = scopes[i].unjump();
                    PartialExpr::Shift(UnionIndex(v.index() + start), 0)
                }
            };
            scopes.pop();
        }
        UnionIndex(root.index() + start)
    }
}
