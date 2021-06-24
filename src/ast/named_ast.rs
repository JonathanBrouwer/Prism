use crate::ast::ast::*;
use crate::ast::base_ast::*;
use std::collections::{HashMap, VecDeque};

#[derive(Copy, Clone, Default)]
pub struct NameReference {
    name: Option<NameOrigin>
}

pub type NamedCollection = AstCollection<NameReference>;

#[derive(Copy, Clone)]
enum NameOrigin {
    Indirect(AstIndex),
    Direct(AstIndex)
}
struct NameEnvironment {
    stack: VecDeque<HashMap<String, NameOrigin>>,
    errors: Vec<AstIndex>
}

impl NameEnvironment {
    fn level_up(&mut self) {
        self.stack.push_back(HashMap::new());
    }
    fn level_down(&mut self) {
        self.stack.pop_back().unwrap();
    }
    fn get(&self, k: String) -> Option<NameOrigin> {
        self.stack.iter().rev().map(|map| map.get(&k)).find(Option::is_some).map(|inner| *inner.unwrap())
    }
    fn insert(&mut self, k: String, v: NameOrigin) {
        self.stack.back_mut().unwrap().insert(k, v);
    }
}

pub fn base_to_named(from: BaseAstCollection) -> Result<NamedCollection, Vec<AstIndex>> {
    let mut env = NameEnvironment { stack: VecDeque::new(), errors: Vec::new() };
    env.level_up();

    let mut to : NamedCollection = from.create_empty_derivative();
    fn sub(ast: AstIndex, env: &mut NameEnvironment, from: &BaseAstCollection, to: &mut NamedCollection) {
        match &from[ast].sub {
            AstSub::DefineId { name, value } => {
                sub(*value, env, from, to);
                env.insert(name.name.clone(), NameOrigin::Direct(*value));
            }
            AstSub::RetrieveId { name } => {
                if let Some(v) = env.get(name.name.clone()) {
                    to[ast] = NameReference { name: Some(v) };
                }else {
                    env.errors.push(ast);
                }
            }
            AstSub::Function { inputs, body } => {
                env.level_up();
                inputs.iter().for_each(|input| env.insert(input.name.clone(), NameOrigin::Indirect(ast)));
                sub(*body, env, from, to);
                env.level_down()
            }
            AstSub::Call { function, args } => {
                sub(*function, env, from, to);
                args.iter().for_each(|arg| sub(*arg, env, from, to));
            }
            AstSub::FunctionType { inputs, output } => {
                inputs.iter().for_each(|inp| sub(*inp, env, from, to));
                sub(*output, env, from, to);
            }
            AstSub::MultType { values } => {
                values.iter().for_each(|v| sub(*v, env, from, to));
            }
            AstSub::AddType { values } => {
                values.iter().for_each(|v| sub(*v, env, from, to));
            }
            AstSub::Case { value, name, cases } => {
                sub(*value, env, from, to);
                env.level_up();
                env.insert(name.name.clone(), NameOrigin::Indirect(ast));
                cases.iter().for_each(|c| sub(*c, env, from, to));
                env.level_down();
            }
            AstSub::Sequence { expressions } => {
                env.level_up();
                expressions.iter().for_each(|e| sub(*e, env, from, to));
                env.level_down();
            }
        };
        to[ast] = NameReference { name: None }
    }
    sub(from.start, &mut env, &from, &mut to);
    Ok(to)
}