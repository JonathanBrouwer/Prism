use crate::lang::{PartialExpr, TcEnv, UnionIndex};
use exhaustive::{DataSourceTaker, Exhaustive};
use std::fmt::{Debug, Formatter};

pub struct ExprWithEnv(pub TcEnv, pub UnionIndex);

impl Exhaustive for ExprWithEnv {
    fn generate(u: &mut DataSourceTaker) -> exhaustive::Result<Self> {
        let mut env = TcEnv::new();
        let idx = arbitrary_rec(0, &mut env, u)?;
        Ok(ExprWithEnv(env, idx))
    }
}

fn arbitrary_rec<'a>(
    scope_size: usize,
    env: &mut TcEnv,
    u: &mut DataSourceTaker,
) -> exhaustive::Result<UnionIndex> {
    let expr = match u.choice(6)? {
        0 => PartialExpr::Type,
        1 if scope_size > 0 => PartialExpr::Var(u.choice(scope_size)?),
        1 if scope_size == 0 => PartialExpr::Type,
        2 => PartialExpr::Free,
        3 => PartialExpr::FnType(
            arbitrary_rec(scope_size, env, u)?,
            arbitrary_rec(scope_size + 1, env, u)?,
        ),
        4 => PartialExpr::FnConstruct(
            arbitrary_rec(scope_size, env, u)?,
            arbitrary_rec(scope_size + 1, env, u)?,
        ),
        5 => PartialExpr::FnDestruct(
            arbitrary_rec(scope_size, env, u)?,
            arbitrary_rec(scope_size, env, u)?,
        ),
        _ => unreachable!(),
    };

    Ok(env.store(expr))
}

impl Debug for ExprWithEnv {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.index_to_string(self.1))
    }
}
