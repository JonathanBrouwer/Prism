use crate::desugar::{Guid, ParseEnv, ParseIndex, SourceExpr};
use crate::lang::error::TypeError;
use crate::lang::{PartialExpr, TcEnv, UnionIndex, ValueOrigin};
use crate::lang::env::GenericEnv;

/// Stores the variables in scope + the depth of the scope
#[derive(Default, Clone)]
struct Scope<'a> {
    names: rpds::RedBlackTreeMap<&'a str, usize>,
    named_scopes: rpds::RedBlackTreeMap<Guid, rpds::RedBlackTreeMap<&'a str, usize>>,
    depth: usize,
}

impl<'a> Scope<'a> {
    pub fn insert_name(&self, key: &'a str) -> Self {
        Scope {
            names: self.names.insert(key, self.depth),
            named_scopes: self.named_scopes.clone(),
            depth: self.depth + 1,
        }
    }

    pub fn insert_jump(&self, guid: Guid) -> Self {
        Scope {
            names: self.names.clone(),
            named_scopes: self.named_scopes.insert(guid, self.names.clone()),
            depth: self.depth,
        }
    }
    
    pub fn jump(&self, guid: Guid) -> Self {
        Scope {
            names: self.named_scopes[&guid].clone(),
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
                    } else if let Some(depth) = scopes[i].names.get(name.as_str()) {
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
                    //TODO make names a stack?
                    //TODO test for multiple depths of syntax: use custom syntax in definition of new syntax
                    todo!()
                }
            };
            scopes.pop();
        }
        UnionIndex(root.index() + start)
    }
}
