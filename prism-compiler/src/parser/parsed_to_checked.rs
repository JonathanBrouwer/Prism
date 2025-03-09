use crate::lang::env::DbEnv;
use crate::lang::error::TypeError;
use crate::lang::{CoreIndex, CorePrismExpr, PrismEnv, ValueOrigin};
use crate::parser::named_env::{NamedEnv, NamesEntry, NamesEnv};
use crate::parser::parse_expr::{GrammarEnvEntry, reduce_expr};
use crate::parser::{ParsedIndex, ParsedPrismExpr};
use prism_parser::core::span::Span;
use prism_parser::parsable::guid::Guid;
use std::collections::HashMap;

impl<'arn, 'grm: 'arn> PrismEnv<'arn, 'grm> {
    pub fn parsed_to_checked(&mut self, i: ParsedIndex) -> CoreIndex {
        self.parsed_to_checked_with_env(i, NamedEnv::default(), &mut HashMap::new())
    }

    pub fn parsed_to_checked_with_env(
        &mut self,
        i: ParsedIndex,
        env: NamedEnv<'arn, 'grm>,
        jump_labels: &mut HashMap<Guid, NamesEnv<'arn, 'grm>>,
    ) -> CoreIndex {
        let e = match self.parsed_values[*i] {
            ParsedPrismExpr::Free => CorePrismExpr::Free,
            ParsedPrismExpr::Type => CorePrismExpr::Type,
            ParsedPrismExpr::Let(n, v, b) => CorePrismExpr::Let(
                self.parsed_to_checked_with_env(v, env, jump_labels),
                self.parsed_to_checked_with_env(
                    b,
                    env.insert_name(n, self.input, self.allocs),
                    jump_labels,
                ),
            ),
            ParsedPrismExpr::FnType(n, a, b) => CorePrismExpr::FnType(
                self.parsed_to_checked_with_env(a, env, jump_labels),
                self.parsed_to_checked_with_env(
                    b,
                    env.insert_name(n, self.input, self.allocs),
                    jump_labels,
                ),
            ),
            ParsedPrismExpr::FnConstruct(n, b) => {
                CorePrismExpr::FnConstruct(self.parsed_to_checked_with_env(
                    b,
                    env.insert_name(n, self.input, self.allocs),
                    jump_labels,
                ))
            }
            ParsedPrismExpr::FnDestruct(f, a) => CorePrismExpr::FnDestruct(
                self.parsed_to_checked_with_env(f, env, jump_labels),
                self.parsed_to_checked_with_env(a, env, jump_labels),
            ),
            ParsedPrismExpr::TypeAssert(v, t) => CorePrismExpr::TypeAssert(
                self.parsed_to_checked_with_env(v, env, jump_labels),
                self.parsed_to_checked_with_env(t, env, jump_labels),
            ),
            ParsedPrismExpr::Name(name) => {
                assert_ne!(name, "_");

                match env.resolve_name_use(name) {
                    Some(NamesEntry::FromEnv(prev_env_len)) => {
                        CorePrismExpr::DeBruijnIndex(env.len() - prev_env_len - 1)
                    }
                    Some(NamesEntry::FromEnvSubst(..)) => {
                        todo!()
                    }
                    Some(NamesEntry::FromParsed(parsed, old_names)) => {
                        if let Some(&expr) = parsed.try_into_value::<ParsedIndex>() {
                            return self.parsed_to_checked_with_env(
                                expr,
                                env.shift_back(old_names, self.input, self.allocs),
                                jump_labels,
                            );
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
                        CorePrismExpr::Free
                    }
                }
            }
            ParsedPrismExpr::ShiftTo(
                b,
                guid,
                captured_env,
                GrammarEnvEntry {
                    grammar_env,
                    common_len,
                },
            ) => {
                let mut names = jump_labels[&guid];
                let original_names = names;

                for (name, value) in captured_env
                    .into_iter()
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                {
                    names = names.insert(
                        name,
                        NamesEntry::FromParsed(reduce_expr(value, self), env.names),
                        self.allocs,
                    );
                }

                let extra_applied_names = grammar_env.len() - common_len;

                let env = NamedEnv::<'arn, 'grm> {
                    env_len: env.env_len + extra_applied_names,
                    names,
                    //TODO should these be preserved?
                    hygienic_names: Default::default(),
                };

                println!("{names:?}");

                let body = self.parsed_to_checked_with_env(b, env, jump_labels);
                return self.apply_names(original_names, grammar_env, common_len, body);
            }
            ParsedPrismExpr::GrammarValue(v, guid) => {
                env.insert_shift_label(guid, jump_labels);
                CorePrismExpr::GrammarValue(v, guid)
            }
            ParsedPrismExpr::GrammarType => CorePrismExpr::GrammarType,
        };
        self.store_checked(e, ValueOrigin::SourceCode(self.parsed_spans[*i]))
    }

    fn apply_names(
        &mut self,
        mut names: NamesEnv<'arn, 'grm>,
        mut grammar_env: DbEnv<'arn>,
        common_len: usize,
        mut body: CoreIndex,
    ) -> CoreIndex {
        while grammar_env.len() > common_len {
            let ((name, names_entry), names_rest) = names.split().unwrap();
            let NamesEntry::FromEnv(prev_env_len) = names_entry else {
                panic!("Got non-names entry")
            };
            names = names_rest;

            let (((), grammar_entry), grammar_rest) = grammar_env.split().unwrap();
            grammar_env = grammar_rest;

            // Always 0 because grammar_env changes
            let db_index = grammar_env.len() - prev_env_len;
            assert_eq!(db_index, 0);

            let typ = self.store_checked(
                CorePrismExpr::Type,
                ValueOrigin::SourceCode(Span::invalid()),
            );
            body = self.store_checked(
                CorePrismExpr::Let(typ, body),
                ValueOrigin::SourceCode(Span::invalid()),
            );
        }

        body
    }
}
