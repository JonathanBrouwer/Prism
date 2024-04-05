use std::fmt::{Debug, Formatter};
use arbitrary::{Arbitrary, Unstructured};
use crate::coc::{PartialExpr, TcEnv, UnionIndex};

pub struct ExprWithEnv(pub TcEnv, pub UnionIndex);

const MAX_SIZE: usize = 16;

impl<'a> Arbitrary<'a> for ExprWithEnv {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let mut env = TcEnv::new();
        let idx = arbitrary_rec(MAX_SIZE, 0, &mut env, u)?;
        Ok(ExprWithEnv(env, idx))
    }
}

fn arbitrary_rec<'a>(depth_left: usize, scope_size: usize, env: &mut TcEnv, u: &mut Unstructured<'a>) -> arbitrary::Result<UnionIndex> {
    let expr = if depth_left == 0 {
        match u.choose_index(2)? {
            0 => PartialExpr::Type,
            1 => PartialExpr::Var(u.choose_index(scope_size + 1)?),
            _ => unreachable!(),
        }
    } else {
        match u.choose_index(2)? {
            0 => PartialExpr::Type,
            1 => PartialExpr::Var(u.choose_index(scope_size + 1)?),
            2 => PartialExpr::FnType(
                arbitrary_rec(depth_left - 1, scope_size, env, u)?,
                arbitrary_rec(depth_left - 1, scope_size + 1, env, u)?
            ),
            3 => PartialExpr::FnConstruct(
                arbitrary_rec(depth_left - 1, scope_size, env, u)?,
                arbitrary_rec(depth_left - 1, scope_size + 1, env, u)?
            ),
            4 => PartialExpr::FnDestruct(
                arbitrary_rec(depth_left - 1, scope_size, env, u)?,
                arbitrary_rec(depth_left - 1, scope_size, env, u)?
            ),
            _ => unreachable!(),
        }
    };
    Ok(env.insert_union_index(expr))
}

impl Debug for ExprWithEnv {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.index_to_string(self.1))
    }
}