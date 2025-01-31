use crate::lang::PrismEnv;
use crate::lang::ValueOrigin;
use crate::lang::env::Env;
use crate::lang::env::EnvEntry::*;
use crate::lang::error::{AggregatedTypeError, TypeError};
use crate::lang::{CheckedIndex, CheckedPrismExpr};
use crate::parser::parse_expr::reduce_expr;
use crate::parser::{ParsedIndex, ParsedPrismExpr};
use prism_parser::core::input::Input;
use prism_parser::parsable::guid::Guid;
use prism_parser::parsable::parsed::Parsed;
use prism_parser::parser::var_map::VarMap;
use rpds::HashTrieMap;
use std::mem;

#[derive(Default, Clone)]
pub struct NamedEnv<'arn, 'grm> {
    env_len: usize,
    names: HashTrieMap<&'arn str, NamesEntry<'arn, 'grm>>,
    jump_labels: HashTrieMap<Guid, HashTrieMap<&'arn str, NamesEntry<'arn, 'grm>>>,
    hygienic_names: HashTrieMap<&'arn str, usize>,
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
    pub fn insert_name(&self, name: &'arn str, input: &'arn str) -> Self {
        let mut s = self.insert_name_at(name, self.env_len, input);
        s.env_len += 1;
        s
    }

    pub fn insert_name_at(&self, name: &'arn str, depth: usize, input: &'arn str) -> Self {
        let names = self.names.insert(name, NamesEntry::FromEnv(depth));
        let hygienic_names = if let Some(NamesEntry::FromParsed(ar, _)) = self.names.get(name) {
            let new_name = ar.into_value::<Input>().as_str(input);
            self.hygienic_names.insert(new_name, depth)
        } else {
            self.hygienic_names.clone()
        };

        Self {
            env_len: self.env_len,
            names,
            jump_labels: self.jump_labels.clone(),
            hygienic_names,
        }
    }

    pub fn resolve_name_use(&self, name: &str) -> Option<&NamesEntry<'arn, 'grm>> {
        self.names.get(name)
    }

    pub fn len(&self) -> usize {
        self.env_len
    }

    pub fn is_empty(&self) -> bool {
        self.env_len == 0
    }

    pub fn insert_shift_label(&self, guid: Guid) -> Self {
        Self {
            env_len: self.env_len,
            names: self.names.clone(),
            jump_labels: self.jump_labels.insert(guid, self.names.clone()),
            hygienic_names: self.hygienic_names.clone(),
        }
    }

    pub fn shift_to_label(
        &self,
        guid: Guid,
        vars: VarMap<'arn, 'grm>,
        env: &mut PrismEnv<'arn, 'grm>,
    ) -> Self {
        let mut names = self.jump_labels[&guid].clone();

        for (name, value) in vars {
            names.insert_mut(
                name,
                NamesEntry::FromParsed(reduce_expr(value, env), self.names.clone()),
            );
        }

        Self {
            env_len: self.env_len,
            names,
            jump_labels: self.jump_labels.clone(),
            //TODO should these be preserved?
            hygienic_names: Default::default(),
        }
    }

    pub fn shift_back(
        &self,
        old_names: &HashTrieMap<&'arn str, NamesEntry<'arn, 'grm>>,
        input: &'arn str,
    ) -> Self {
        let mut new_env = Self {
            env_len: self.env_len,
            names: old_names.clone(),
            jump_labels: self.jump_labels.clone(),
            //TODO what here? old code takes from `old_names` env (not available here)
            hygienic_names: Default::default(),
        };

        for (name, db_idx) in &self.hygienic_names {
            new_env = new_env.insert_name_at(name, *db_idx, input);
        }

        new_env
    }
}

impl<'arn, 'grm: 'arn> PrismEnv<'arn, 'grm> {
    pub fn type_check(
        &mut self,
        root: ParsedIndex,
    ) -> Result<(CheckedIndex, CheckedIndex), AggregatedTypeError> {
        let root = self.parsed_to_checked(root, &NamedEnv::default());
        let ti = self._type_check(root, &Env::default());

        let errors = mem::take(&mut self.errors);
        if errors.is_empty() {
            Ok((root, ti))
        } else {
            Err(AggregatedTypeError { errors })
        }
    }

    fn parsed_to_checked(&mut self, i: ParsedIndex, env: &NamedEnv<'arn, 'grm>) -> CheckedIndex {
        let e = match self.parsed_values[*i] {
            ParsedPrismExpr::Free => CheckedPrismExpr::Free,
            ParsedPrismExpr::Type => CheckedPrismExpr::Type,
            ParsedPrismExpr::Let(n, v, b) => CheckedPrismExpr::Let(
                self.parsed_to_checked(v, env),
                self.parsed_to_checked(b, &env.insert_name(n, self.input)),
            ),
            ParsedPrismExpr::FnType(n, a, b) => CheckedPrismExpr::FnType(
                self.parsed_to_checked(a, env),
                self.parsed_to_checked(b, &env.insert_name(n, self.input)),
            ),
            ParsedPrismExpr::FnConstruct(n, b) => CheckedPrismExpr::FnConstruct(
                self.parsed_to_checked(b, &env.insert_name(n, self.input)),
            ),
            ParsedPrismExpr::FnDestruct(f, a) => CheckedPrismExpr::FnDestruct(
                self.parsed_to_checked(f, env),
                self.parsed_to_checked(a, env),
            ),
            ParsedPrismExpr::TypeAssert(v, t) => CheckedPrismExpr::TypeAssert(
                self.parsed_to_checked(v, env),
                self.parsed_to_checked(t, env),
            ),
            ParsedPrismExpr::Name(name) => {
                assert_ne!(name, "_");

                match env.resolve_name_use(name) {
                    Some(NamesEntry::FromEnv(prev_env_len)) => {
                        CheckedPrismExpr::DeBruijnIndex(env.len() - *prev_env_len - 1)
                    }
                    Some(NamesEntry::FromParsed(parsed, old_names)) => {
                        if let Some(&expr) = parsed.try_into_value::<ParsedIndex>() {
                            return self
                                .parsed_to_checked(expr, &env.shift_back(old_names, self.input));
                        } else if let Some(_name) = parsed.try_into_value::<Input>() {
                            todo!()
                            // self.values[*i] = PrismExpr::Name(name.as_str(self.input));
                            // self._type_check(i, env)
                        } else {
                            unreachable!(
                                "Found name `{name}` referring to {}",
                                parsed.to_debug_string(self.input)
                            );
                        }
                    }
                    None => {
                        self.errors
                            .push(TypeError::UnknownName(self.parsed_spans[*i]));
                        CheckedPrismExpr::Free
                    }
                }
            }
            ParsedPrismExpr::ShiftLabel(b, guid) => {
                return self.parsed_to_checked(b, &env.insert_shift_label(guid));
            }
            ParsedPrismExpr::ShiftTo(b, guid, captured_env) => {
                let env = env.shift_to_label(guid, captured_env, self);
                return self.parsed_to_checked(b, &env);
            }
            ParsedPrismExpr::ParserValue(v) => CheckedPrismExpr::ParserValue(v),
            ParsedPrismExpr::ParserValueType => CheckedPrismExpr::ParserValueType,
        };
        self.store_checked(e, ValueOrigin::SourceCode(self.parsed_spans[*i]))
    }

    /// Type checkes `i` in scope `s`. Returns the type.
    /// Invariant: Returned UnionIndex is valid in Env `s`
    fn _type_check(&mut self, i: CheckedIndex, env: &Env) -> CheckedIndex {
        // We should only type check values from the source code
        assert!(matches!(
            self.checked_origins[*i],
            ValueOrigin::SourceCode(_)
        ));

        let t = match self.checked_values[*i] {
            CheckedPrismExpr::Type => CheckedPrismExpr::Type,
            CheckedPrismExpr::Let(mut v, b) => {
                // Check `v`
                let err_count = self.errors.len();
                let vt = self._type_check(v, env);
                if self.errors.len() > err_count {
                    v = self.store_checked(CheckedPrismExpr::Free, ValueOrigin::Failure);
                }

                let bt = self._type_check(b, &env.cons(CSubst(v, vt)));
                CheckedPrismExpr::Let(v, bt)
            }
            CheckedPrismExpr::DeBruijnIndex(index) => match env.get(index) {
                Some(&CType(_, t) | &CSubst(_, t)) => CheckedPrismExpr::Shift(t, index + 1),
                Some(_) => unreachable!(),
                None => {
                    self.errors.push(TypeError::IndexOutOfBound(i));
                    return self.store_checked(CheckedPrismExpr::Free, ValueOrigin::Failure);
                }
            },
            CheckedPrismExpr::FnType(mut a, b) => {
                let err_count = self.errors.len();
                let at = self._type_check(a, env);
                self.expect_beq_type(at, env);
                if self.errors.len() > err_count {
                    a = self.store_checked(CheckedPrismExpr::Free, ValueOrigin::Failure);
                }

                let err_count = self.errors.len();
                let bs = env.cons(CType(self.new_tc_id(), a));
                let bt = self._type_check(b, &bs);

                // Check if `b` typechecked without errors.
                if self.errors.len() == err_count {
                    self.expect_beq_type(bt, &bs);
                }

                CheckedPrismExpr::Type
            }
            CheckedPrismExpr::FnConstruct(b) => {
                let a = self.store_checked(CheckedPrismExpr::Free, ValueOrigin::FreeSub(i));
                let bs = env.cons(CType(self.new_tc_id(), a));
                let bt = self._type_check(b, &bs);
                CheckedPrismExpr::FnType(a, bt)
            }
            CheckedPrismExpr::FnDestruct(f, mut a) => {
                let err_count = self.errors.len();
                let at = self._type_check(a, env);
                if self.errors.len() > err_count {
                    a = self.store_checked(CheckedPrismExpr::Free, ValueOrigin::Failure);
                };

                let rt = self.store_checked(CheckedPrismExpr::Free, ValueOrigin::TypeOf(i));

                let err_count = self.errors.len();
                let ft = self._type_check(f, env);
                if self.errors.len() == err_count {
                    self.expect_beq_fn_type(ft, at, rt, env)
                }

                CheckedPrismExpr::Let(a, rt)
            }
            CheckedPrismExpr::TypeAssert(e, typ) => {
                let err_count1 = self.errors.len();
                let et = self._type_check(e, env);

                let err_count2 = self.errors.len();
                let typt = self._type_check(typ, env);
                if self.errors.len() == err_count2 {
                    self.expect_beq_type(typt, env);
                }

                if self.errors.len() == err_count1 {
                    self.expect_beq_assert(e, et, typ, env);
                }

                return et;
            }
            CheckedPrismExpr::Free => {
                // TODO self.queued_tc.insert(i, (s.clone(), t));
                CheckedPrismExpr::Free
            }
            CheckedPrismExpr::Shift(v, shift) => {
                assert_eq!(shift, 0);
                return self._type_check(v, env);

                //TODO
                // PrismExpr::Shift(self._type_check(v, &env.shift(shift)), shift)
            }
            CheckedPrismExpr::ParserValue(_) => CheckedPrismExpr::ParserValueType,
            CheckedPrismExpr::ParserValueType => CheckedPrismExpr::Type,
        };
        let tid = self.store_checked(t, ValueOrigin::TypeOf(i));
        self.checked_types.insert(i, tid);
        tid
    }
}
