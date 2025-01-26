use crate::lang::UnionIndex;
use crate::lang::ValueOrigin;
use crate::lang::env::EnvEntry::*;
use crate::lang::env::{Env, EnvEntry};
use crate::lang::error::{AggregatedTypeError, TypeError};
use crate::lang::expect_beq::GENERATED_NAME;
use crate::lang::{PrismEnv, PrismExpr};
use prism_parser::core::input::Input;
use prism_parser::parsable::guid::Guid;
use prism_parser::parsable::parsed::Parsed;
use prism_parser::parser::var_map::VarMap;
use rpds::HashTrieMap;
use std::collections::HashMap;
use std::mem;

#[derive(Default, Clone)]
pub struct NamedEnv<'arn, 'grm> {
    env: Env,
    names: HashTrieMap<&'arn str, NamesEntry<'arn, 'grm>>,
    jump_labels: HashTrieMap<Guid, HashTrieMap<&'arn str, NamesEntry<'arn, 'grm>>>,
}

#[derive(Debug)]
pub enum NamesEntry<'arn, 'grm> {
    FromEnv(usize),
    FromParsed(
        Parsed<'arn, 'grm>,
        HashTrieMap<&'arn str, NamesEntry<'arn, 'grm>>,
    ),
}

impl<'arn, 'grm: 'arn> NamedEnv<'arn, 'grm> {
    pub fn cons(&self, name: &'arn str, value: EnvEntry) -> Self {
        Self {
            env: self.env.cons(value),
            names: self.names.insert(name, NamesEntry::FromEnv(self.env.len())),
            jump_labels: self.jump_labels.clone(),
        }
    }

    pub fn resole_de_bruijn_idx(&self, index: usize) -> Option<&EnvEntry> {
        self.env.get(index)
    }

    pub fn resolve_name_use(&self, name: &str) -> Option<&NamesEntry<'arn, 'grm>> {
        self.names.get(name)
    }

    pub fn resolve_name_decl(&self, name: &'arn str, input: &'arn str) -> &'arn str {
        match self.names.get(name) {
            None | Some(NamesEntry::FromEnv(_)) => name,
            Some(NamesEntry::FromParsed(parsed, new_names)) => {
                if let Some(new_name) = parsed.try_into_value::<Input>() {
                    new_name.as_str(input)
                } else {
                    unreachable!()
                }
            }
        }
    }

    pub fn len(&self) -> usize {
        self.env.len()
    }

    pub fn insert_shift_label(&self, guid: Guid) -> Self {
        Self {
            env: self.env.clone(),
            names: self.names.clone(),
            jump_labels: self.jump_labels.insert(guid, self.names.clone()),
        }
    }

    pub fn shift_to_label(&self, guid: Guid, vars: VarMap<'arn, 'grm>) -> Self {
        let mut names = self.jump_labels[&guid].clone();

        for (name, value) in vars {
            names.insert_mut(name, NamesEntry::FromParsed(value, self.names.clone()));
        }

        Self {
            env: self.env.clone(),
            names,
            jump_labels: self.jump_labels.clone(),
        }
    }

    pub fn shift_back(&self, old_names: &HashTrieMap<&'arn str, NamesEntry<'arn, 'grm>>) -> Self {
        println!("{:?}", old_names.keys().collect::<Vec<_>>());

        println!("{:?}", &self.names.keys().collect::<Vec<_>>());

        Self {
            env: self.env.clone(),
            names: old_names.clone(),
            jump_labels: self.jump_labels.clone(),
        }
    }
}

impl<'arn, 'grm: 'arn> PrismEnv<'arn, 'grm> {
    pub fn type_check(&mut self, root: UnionIndex) -> Result<UnionIndex, AggregatedTypeError> {
        let ti = self._type_check(root, &NamedEnv::default());

        let errors = mem::take(&mut self.errors);
        if errors.is_empty() {
            Ok(ti)
        } else {
            Err(AggregatedTypeError { errors })
        }
    }

    /// Type checkes `i` in scope `s`. Returns the type.
    /// Invariant: Returned UnionIndex is valid in Env `s`
    fn _type_check(&mut self, i: UnionIndex, env: &NamedEnv<'arn, 'grm>) -> UnionIndex {
        // We should only type check values from the source code
        assert!(matches!(self.value_origins[*i], ValueOrigin::SourceCode(_)));

        let t = match self.values[*i] {
            PrismExpr::Type => PrismExpr::Type,
            PrismExpr::Let(n, mut v, b) => {
                let n = env.resolve_name_decl(n, self.input);

                // Check `v`
                let err_count = self.errors.len();
                let vt = self._type_check(v, env);
                if self.errors.len() > err_count {
                    v = self.store(PrismExpr::Free, ValueOrigin::Failure);
                }

                let bt = self._type_check(b, &env.cons(n, CSubst(v, vt)));
                PrismExpr::Let(n, v, bt)
            }
            PrismExpr::DeBruijnIndex(index) => match env.resole_de_bruijn_idx(index) {
                Some(&CType(_, t) | &CSubst(_, t)) => PrismExpr::Shift(t, index + 1),
                Some(_) => unreachable!(),
                None => {
                    self.errors.push(TypeError::IndexOutOfBound(i));
                    return self.store(PrismExpr::Free, ValueOrigin::Failure);
                }
            },
            PrismExpr::Name(name) => {
                return match env.resolve_name_use(name) {
                    Some(NamesEntry::FromEnv(prev_env_len)) => {
                        self.values[*i] = PrismExpr::DeBruijnIndex(env.len() - *prev_env_len - 1);
                        self._type_check(i, env)
                    }
                    Some(NamesEntry::FromParsed(parsed, old_names)) => {
                        if let Some(expr) = parsed.try_into_value::<UnionIndex>() {
                            self.values[*i] = PrismExpr::Shift(*expr, 0);
                            self._type_check(i, &env.shift_back(old_names))
                        } else if let Some(name) = parsed.try_into_value::<Input>() {
                            self.values[*i] = PrismExpr::Name(name.as_str(self.input));
                            self._type_check(i, env)
                        } else {
                            unreachable!()
                        }
                    }
                    None => {
                        println!("NAME NOT FOUND: {name}");
                        self.errors.push(TypeError::UnknownName(
                            self.value_origins[*i].to_source_span(),
                        ));
                        self.store(PrismExpr::Free, ValueOrigin::Failure)
                    }
                };
            }
            PrismExpr::FnType(n, mut a, b) => {
                let n = env.resolve_name_decl(n, self.input);

                let err_count = self.errors.len();
                let at = self._type_check(a, env);
                self.expect_beq_type(at, &env.env);
                if self.errors.len() > err_count {
                    a = self.store(PrismExpr::Free, ValueOrigin::Failure);
                }

                let err_count = self.errors.len();
                let bs = env.cons(n, CType(self.new_tc_id(), a));
                let bt = self._type_check(b, &bs);

                // Check if `b` typechecked without errors.
                if self.errors.len() == err_count {
                    self.expect_beq_type(bt, &bs.env);
                }

                PrismExpr::Type
            }
            PrismExpr::FnConstruct(n, b) => {
                let n = env.resolve_name_decl(n, self.input);

                let a = self.store(PrismExpr::Free, ValueOrigin::FreeSub(i));
                let bs = env.cons(n, CType(self.new_tc_id(), a));
                let bt = self._type_check(b, &bs);
                PrismExpr::FnType(n, a, bt)
            }
            PrismExpr::FnDestruct(f, mut a) => {
                let err_count = self.errors.len();
                let at = self._type_check(a, env);
                if self.errors.len() > err_count {
                    a = self.store(PrismExpr::Free, ValueOrigin::Failure);
                };

                let rt = self.store(PrismExpr::Free, ValueOrigin::TypeOf(i));

                let err_count = self.errors.len();
                let ft = self._type_check(f, env);
                if self.errors.len() == err_count {
                    self.expect_beq_fn_type(ft, at, rt, &env.env)
                }

                PrismExpr::Let(GENERATED_NAME, a, rt)
            }
            PrismExpr::TypeAssert(e, typ) => {
                let err_count1 = self.errors.len();
                let et = self._type_check(e, env);

                let err_count2 = self.errors.len();
                let typt = self._type_check(typ, env);
                if self.errors.len() == err_count2 {
                    self.expect_beq_type(typt, &env.env);
                }

                if self.errors.len() == err_count1 {
                    self.expect_beq_assert(e, et, typ, &env.env);
                }

                return et;
            }
            PrismExpr::Free => {
                // TODO self.queued_tc.insert(i, (s.clone(), t));
                PrismExpr::Free
            }
            PrismExpr::Shift(v, shift) => {
                assert_eq!(shift, 0);
                return self._type_check(v, env);

                //TODO
                // PrismExpr::Shift(self._type_check(v, &env.shift(shift)), shift)
            }
            PrismExpr::ShiftLabel(b, guid) => {
                self.values[*i] = PrismExpr::Shift(b, 0);
                return self._type_check(i, &env.insert_shift_label(guid));
            }
            PrismExpr::ShiftTo(b, guid, captured_env) => {
                return self._type_check(b, &env.shift_to_label(guid, captured_env));
            }
            PrismExpr::ParserValue(_) => PrismExpr::ParserValueType,
            PrismExpr::ParserValueType => PrismExpr::Type,
        };
        let tid = self.store(t, ValueOrigin::TypeOf(i));
        self.value_types.insert(i, tid);
        tid
    }
}
