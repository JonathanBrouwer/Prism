use crate::desugar::{ParseEnv, ParseIndex, SourceExpr};
use crate::lang::{PartialExpr, TcEnv, UnionIndex, ValueOrigin};
use crate::lang::error::TypeError;

/// Stores the variables in scope + the depth of the scope
#[derive(Default, Clone)]
struct Scope<'a>(rpds::RedBlackTreeMap<&'a str, usize>, usize);

impl<'a> Scope<'a> {
    pub fn insert(&self, key: &'a str) -> Self {
        Scope(self.0.insert(key, self.1), self.1 + 1)
    }
}

impl TcEnv {
    pub fn insert_parse_env(&mut self, parse_env: &ParseEnv, root: ParseIndex) -> UnionIndex {
        let mut names = vec![Scope::default(); parse_env.values().len()];
        
        let start = self.values.len();
        let take_count = root.index() + 1;
        
        self.values.resize(start + take_count, PartialExpr::Free);
        self.value_origins.resize(start + take_count, ValueOrigin::Failure);
        
        for (i, (expr, span)) in parse_env.values().iter().zip(parse_env.value_spans().iter()).take(take_count).enumerate().rev() {
            self.value_origins[start+i] = ValueOrigin::SourceCode(*span);
            self.values[start+i] = match expr {
                SourceExpr::Type => {
                    PartialExpr::Type
                }
                SourceExpr::Let(name, value, body) => {
                    names[value.index()] = names[i].clone();
                    names[body.index()] = names[i].insert(&name);
                    PartialExpr::Let(UnionIndex(value.index() + start), UnionIndex(body.index() + start))
                }
                SourceExpr::Variable(name) => {
                    if name == "_" {
                        PartialExpr::Free
                    } else {
                        if let Some(depth) = names[i].0.get(name.as_str()) {
                            PartialExpr::DeBruijnIndex(names[i].1 - depth - 1)
                        } else {
                            self.errors.push(TypeError::UnknownName(*span));
                            PartialExpr::Free
                        }
                    }
                }
                SourceExpr::FnType(name, arg_type, return_type) => {
                    names[arg_type.index()] = names[i].clone();
                    names[return_type.index()] = names[i].insert(&name);
                    PartialExpr::FnType(UnionIndex(arg_type.index() + start), UnionIndex(return_type.index() + start))
                }
                SourceExpr::FnConstruct(name, arg_type, body) => {
                    names[arg_type.index()] = names[i].clone();
                    names[body.index()] = names[i].insert(&name);
                    PartialExpr::FnConstruct(UnionIndex(arg_type.index() + start), UnionIndex(body.index() + start))
                }
                SourceExpr::FnDestruct(function, arg) => {
                    names[function.index()] = names[i].clone();
                    names[arg.index()] = names[i].clone();
                    PartialExpr::FnDestruct(UnionIndex(function.index() + start), UnionIndex(arg.index() + start))
                }
            };
            names.pop();
        }
        UnionIndex(root.index() + start)
    }
}