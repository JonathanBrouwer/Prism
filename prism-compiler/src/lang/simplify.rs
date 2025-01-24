use crate::lang::env::{Env, EnvEntry, UniqueVariableId};
use crate::lang::UnionIndex;
use crate::lang::{PrismEnv, PrismExpr};
use std::collections::HashMap;

impl<'arn, 'grm: 'arn> PrismEnv<'arn, 'grm> {
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
            PrismExpr::Type => PrismExpr::Type,
            PrismExpr::Let(n, v, b) => {
                let v = self.simplify_inner(v, s, var_map);
                let id = self.new_tc_id();
                var_map.insert(id, var_map.len());
                let b = self.simplify_inner(b, &s.cons(EnvEntry::RType(id)), var_map);
                var_map.remove(&id);
                PrismExpr::Let(n, v, b)
            }
            PrismExpr::DeBruijnIndex(v) => match s.get(v) {
                Some(EnvEntry::CType(_, _, _)) | Some(EnvEntry::CSubst(_, _, _)) => unreachable!(),
                Some(EnvEntry::RType(id)) => {
                    PrismExpr::DeBruijnIndex(var_map.len() - var_map[id] - 1)
                }
                Some(EnvEntry::RSubst(subst, subst_env)) => {
                    return self.simplify_inner(*subst, subst_env, var_map)
                }
                None => PrismExpr::DeBruijnIndex(v),
            },
            PrismExpr::FnType(n, a, b) => {
                let a = self.simplify_inner(a, s, var_map);
                let id = self.new_tc_id();
                var_map.insert(id, var_map.len());
                let b = self.simplify_inner(b, &s.cons(EnvEntry::RType(id)), var_map);
                var_map.remove(&id);
                PrismExpr::FnType(n, a, b)
            }
            PrismExpr::FnConstruct(n, b) => {
                let id = self.new_tc_id();
                var_map.insert(id, var_map.len());
                let b = self.simplify_inner(b, &s.cons(EnvEntry::RType(id)), var_map);
                var_map.remove(&id);
                PrismExpr::FnConstruct(n, b)
            }
            PrismExpr::FnDestruct(a, b) => {
                let a = self.simplify_inner(a, s, var_map);
                let b = self.simplify_inner(b, s, var_map);
                PrismExpr::FnDestruct(a, b)
            }
            PrismExpr::Free => PrismExpr::Free,
            PrismExpr::Shift(b, i) => {
                return self.simplify_inner(b, &s.shift(i.min(s.len())), var_map)
            }
            PrismExpr::TypeAssert(e, typ) => {
                let e = self.simplify_inner(e, s, var_map);
                let typ = self.simplify_inner(typ, s, var_map);
                PrismExpr::TypeAssert(e, typ)
            }
            PrismExpr::Name(..)
            | PrismExpr::ShiftPoint(..)
            | PrismExpr::ShiftTo(..)
            | PrismExpr::ParserValue(..) => {
                unreachable!("Should not occur in typechecked terms")
            }
        };
        self.store(e_new, self.value_origins[*i])
    }
}
