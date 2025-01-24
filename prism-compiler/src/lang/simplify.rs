use crate::lang::env::{Env, EnvEntry, UniqueVariableId};
use crate::lang::UnionIndex;
use crate::lang::{PartialExpr, TcEnv};
use std::collections::HashMap;

impl TcEnv<'_> {
    pub fn simplify(&mut self, i: UnionIndex) -> UnionIndex {
        self.simplify_inner(i, &Env::new(), &mut HashMap::new())
    }

    fn simplify_inner(
        &mut self,
        i: UnionIndex,
        s: &Env,
        var_map: &mut HashMap<UniqueVariableId, usize>,
    ) -> UnionIndex {
        let e_new = match self.values[*i] {
            PartialExpr::Type => PartialExpr::Type,
            PartialExpr::Let(n, v, b) => {
                let v = self.simplify_inner(v, s, var_map);
                let id = self.new_tc_id();
                var_map.insert(id, var_map.len());
                let b = self.simplify_inner(b, &s.cons(EnvEntry::RType(id)), var_map);
                var_map.remove(&id);
                PartialExpr::Let(n, v, b)
            }
            PartialExpr::DeBruijnIndex(v) => match s.get(v) {
                Some(EnvEntry::CType(_, _, _)) | Some(EnvEntry::CSubst(_, _, _)) => unreachable!(),
                Some(EnvEntry::RType(id)) => {
                    PartialExpr::DeBruijnIndex(var_map.len() - var_map[id] - 1)
                }
                Some(EnvEntry::RSubst(subst, subst_env)) => {
                    return self.simplify_inner(*subst, subst_env, var_map)
                }
                None => PartialExpr::DeBruijnIndex(v),
            },
            PartialExpr::FnType(n, a, b) => {
                let a = self.simplify_inner(a, s, var_map);
                let id = self.new_tc_id();
                var_map.insert(id, var_map.len());
                let b = self.simplify_inner(b, &s.cons(EnvEntry::RType(id)), var_map);
                var_map.remove(&id);
                PartialExpr::FnType(n, a, b)
            }
            PartialExpr::FnConstruct(n, b) => {
                let id = self.new_tc_id();
                var_map.insert(id, var_map.len());
                let b = self.simplify_inner(b, &s.cons(EnvEntry::RType(id)), var_map);
                var_map.remove(&id);
                PartialExpr::FnConstruct(n, b)
            }
            PartialExpr::FnDestruct(a, b) => {
                let a = self.simplify_inner(a, s, var_map);
                let b = self.simplify_inner(b, s, var_map);
                PartialExpr::FnDestruct(a, b)
            }
            PartialExpr::Free => PartialExpr::Free,
            PartialExpr::Shift(b, i) => {
                return self.simplify_inner(b, &s.shift(i.min(s.len())), var_map)
            }
            PartialExpr::TypeAssert(e, typ) => {
                let e = self.simplify_inner(e, s, var_map);
                let typ = self.simplify_inner(typ, s, var_map);
                PartialExpr::TypeAssert(e, typ)
            }
            PartialExpr::Name(_) | PartialExpr::ShiftPoint(_, _) | PartialExpr::ShiftTo(_, _) => {
                unreachable!("Should not occur in typechecked terms")
            }
        };
        self.store(e_new, self.value_origins[*i])
    }
}
