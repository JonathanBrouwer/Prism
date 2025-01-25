use crate::lang::UnionIndex;
use crate::lang::ValueOrigin;
use crate::lang::env::Env;
use crate::lang::env::EnvEntry::*;
use crate::lang::error::{AggregatedTypeError, TypeError};
use crate::lang::expect_beq::GENERATED_NAME;
use crate::lang::{PrismEnv, PrismExpr};
use prism_parser::parsable::parsed::Parsed;
use rpds::HashTrieMap;
use std::mem;

pub enum NamesEntry<'arn, 'grm> {
    FromEnv(usize),
    FromParsed(Parsed<'arn, 'grm>),
}

impl<'arn, 'grm: 'arn> PrismEnv<'arn, 'grm> {
    pub fn type_check(&mut self, root: UnionIndex) -> Result<UnionIndex, AggregatedTypeError> {
        let ti = self._type_check(root, &Env::new(), &HashTrieMap::new());

        let errors = mem::take(&mut self.errors);
        if errors.is_empty() {
            Ok(ti)
        } else {
            Err(AggregatedTypeError { errors })
        }
    }

    /// Type checkes `i` in scope `s`. Returns the type.
    /// Invariant: Returned UnionIndex is valid in Env `s`
    fn _type_check(
        &mut self,
        i: UnionIndex,
        env: &Env,
        names: &HashTrieMap<&'arn str, NamesEntry<'arn, 'grm>>,
    ) -> UnionIndex {
        // We should only type check values from the source code
        assert!(matches!(self.value_origins[*i], ValueOrigin::SourceCode(_)));

        let t = match self.values[*i] {
            PrismExpr::Type => PrismExpr::Type,
            PrismExpr::Let(n, mut v, b) => {
                let n = self.resolve_name(n, names);

                // Check `v`
                let err_count = self.errors.len();
                let vt = self._type_check(v, env, names);
                if self.errors.len() > err_count {
                    v = self.store(PrismExpr::Free, ValueOrigin::Failure);
                }

                let bt = self._type_check(
                    b,
                    &env.cons(CSubst(v, vt)),
                    &names.insert(n, NamesEntry::FromEnv(env.len())),
                );
                PrismExpr::Let(n, v, bt)
            }
            PrismExpr::DeBruijnIndex(index) => match env.get(index) {
                Some(&CType(_, t) | &CSubst(_, t)) => PrismExpr::Shift(t, index + 1),
                Some(_) => unreachable!(),
                None => {
                    self.errors.push(TypeError::IndexOutOfBound(i));
                    return self.store(PrismExpr::Free, ValueOrigin::Failure);
                }
            },
            PrismExpr::Name(n) => {
                let n = self.resolve_name(n, names);

                return match names.get(n) {
                    Some(NamesEntry::FromEnv(prev_env_len)) => {
                        self.values[*i] = PrismExpr::DeBruijnIndex(env.len() - *prev_env_len - 1);
                        return self._type_check(i, env, names);
                    }
                    Some(NamesEntry::FromParsed(parsed)) => {
                        todo!()
                    }
                    None => {
                        self.errors.push(TypeError::UnknownName(
                            self.value_origins[*i].to_source_span(),
                        ));
                        self.store(PrismExpr::Free, ValueOrigin::Failure)
                    }
                };
                // let n = self.resolve_name(n, s);

                // return if let Some((db_index, _)) = s
                //     .iter()
                //     .enumerate()
                //     .find(|(_, entry)| entry.get_name() == n)
                // {
                //     self.values[*i] = PrismExpr::DeBruijnIndex(db_index);
                //     self._type_check(i, s)
                // } else {

                // };
            }
            PrismExpr::FnType(n, mut a, b) => {
                let n = self.resolve_name(n, names);

                let err_count = self.errors.len();
                let at = self._type_check(a, env, names);
                self.expect_beq_type(at, env);
                if self.errors.len() > err_count {
                    a = self.store(PrismExpr::Free, ValueOrigin::Failure);
                }

                let err_count = self.errors.len();
                let bs = env.cons(CType(self.new_tc_id(), a));
                let bt = self._type_check(b, &bs, &names.insert(n, NamesEntry::FromEnv(env.len())));

                // Check if `b` typechecked without errors.
                if self.errors.len() == err_count {
                    self.expect_beq_type(bt, &bs);
                }

                PrismExpr::Type
            }
            PrismExpr::FnConstruct(n, b) => {
                let n = self.resolve_name(n, names);

                let a = self.store(PrismExpr::Free, ValueOrigin::FreeSub(i));
                let bs: Env = env.cons(CType(self.new_tc_id(), a));
                let bt = self._type_check(b, &bs, &names.insert(n, NamesEntry::FromEnv(env.len())));
                PrismExpr::FnType(n, a, bt)
            }
            PrismExpr::FnDestruct(f, mut a) => {
                let err_count = self.errors.len();
                let at = self._type_check(a, env, names);
                if self.errors.len() > err_count {
                    a = self.store(PrismExpr::Free, ValueOrigin::Failure);
                };

                let rt = self.store(PrismExpr::Free, ValueOrigin::TypeOf(i));

                let err_count = self.errors.len();
                let ft = self._type_check(f, env, names);
                if self.errors.len() == err_count {
                    self.expect_beq_fn_type(ft, at, rt, env)
                }

                PrismExpr::Let(GENERATED_NAME, a, rt)
            }
            PrismExpr::TypeAssert(e, typ) => {
                let err_count1 = self.errors.len();
                let et = self._type_check(e, env, names);

                let err_count2 = self.errors.len();
                let typt = self._type_check(typ, env, names);
                if self.errors.len() == err_count2 {
                    self.expect_beq_type(typt, env);
                }

                if self.errors.len() == err_count1 {
                    self.expect_beq_assert(e, et, typ, env);
                }

                return et;
            }
            PrismExpr::Free => {
                // TODO self.queued_tc.insert(i, (s.clone(), t));
                PrismExpr::Free
            }
            PrismExpr::Shift(v, shift) => {
                assert_eq!(shift, 0);
                return self._type_check(v, env, names);

                //TODO
                // PrismExpr::Shift(self._type_check(v, &env.shift(shift)), shift)
            }
            PrismExpr::ShiftPoint(b, guid) => {
                self.values[*i] = PrismExpr::Shift(b, 0);
                self.guid_shifts.insert(guid, names.clone());
                return self._type_check(i, env, names);
            }
            PrismExpr::ShiftTo(b, guid, captured_env) => {
                let prev_names = self.guid_shifts[&guid].clone();

                return self._type_check(b, env, &prev_names);

                todo!()
                // let prev_len = self.guid_shifts[&guid];
                // let mut s = s.clone();
                //
                // let mut b: PrismExpr = PrismExpr::Shift(b, s.len() - prev_len);
                // for (name, value) in env {
                //     //TODO name
                //     s = s.cons(PSubst(value, "_"));
                // }
                //
                // self.values[*i] = b;
                // return self._type_check(i, &s);

                //
                // let mut b: PrismExpr = PrismExpr::ShiftToTrigger(b, prev_len, s.len());
                //
                // for (name, value) in env {
                //     let value = if let Some(value) = value.try_into_value::<UnionIndex>() {
                //         *value
                //     } else {
                //         self.store(PrismExpr::ParserValue(value), self.value_origins[*i])
                //     };
                //     let old_b = self.store(b, self.value_origins[*i]);
                //     b = PrismExpr::Let(name, value, old_b);
                // }
                //
                // self.values[*i] = b;
                // return self._type_check(i, s);
            }
            // PrismExpr::ShiftToTrigger(b, s_from, s_to) => {
            //     self.values[*i] = PrismExpr::Shift(b, 0);
            //     return self._type_check(
            //         i,
            //         &s.fill_range(
            //             s_from..s_to,
            //             CSubst(UnionIndex(usize::MAX), UnionIndex(usize::MAX), "_"),
            //         ),
            //     );
            // }
            PrismExpr::ParserValue(_) => PrismExpr::ParserValueType,
            PrismExpr::ParserValueType => PrismExpr::Type,
        };
        let tid = self.store(t, ValueOrigin::TypeOf(i));
        self.value_types.insert(i, tid);
        tid
    }

    fn resolve_name(
        &mut self,
        name: &'arn str,
        names: &HashTrieMap<&'arn str, NamesEntry<'arn, 'grm>>,
    ) -> &'arn str {
        name
    }
}
