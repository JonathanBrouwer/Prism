use crate::lang::error::TypeError;
use crate::lang::{CoreIndex, CorePrismExpr, PrismEnv, ValueOrigin};
use crate::parser::named_env::{NamedEnv, NamesEntry, NamesEnv};
use crate::parser::{ParsedIndex, ParsedPrismExpr};
use prism_parser::grammar::grammar_file::GrammarFile;
use std::collections::HashMap;

impl<'arn, 'grm: 'arn> PrismEnv<'arn, 'grm> {
    pub fn parsed_to_checked(&mut self, i: ParsedIndex) -> CoreIndex {
        self.parsed_to_checked_with_env(i, NamedEnv::default(), &mut Default::default())
    }

    pub fn parsed_to_checked_with_env(
        &mut self,
        i: ParsedIndex,
        env: NamedEnv<'arn, 'grm>,
        jump_labels: &mut HashMap<*const GrammarFile<'arn, 'grm>, NamesEnv<'arn, 'grm>>,
    ) -> CoreIndex {
        let origin = ValueOrigin::SourceCode(self.parsed_spans[*i]);
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
                    Some(NamesEntry::FromGrammarEnv {
                        adapt_env_len,
                        prev_env_len,
                    }) => {
                        let adapt_env_len = adapt_env_len - 1;
                        let grammar_expr = self.store_checked(
                            CorePrismExpr::DeBruijnIndex(env.len() - adapt_env_len - 1),
                            origin,
                        );

                        // println!("{adapt_env_len} {prev_env_len}");
                        let idx = prev_env_len + 1;
                        let e = self.store_checked(CorePrismExpr::DeBruijnIndex(idx), origin);
                        let mut e = self.store_checked(CorePrismExpr::FnConstruct(e), origin);
                        for _ in 0..adapt_env_len {
                            e = self.store_checked(CorePrismExpr::FnConstruct(e), origin);
                        }
                        CorePrismExpr::FnDestruct(grammar_expr, e)
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
            ParsedPrismExpr::GrammarType => CorePrismExpr::GrammarType,
            ParsedPrismExpr::GrammarValue(grammar) => {
                env.insert_shift_label(grammar, jump_labels);

                // Create \f. f g [env0] [env1] ...
                let mut e = self.store_checked(CorePrismExpr::DeBruijnIndex(0), origin);
                for i in 0..env.len() {
                    let v = self.store_checked(CorePrismExpr::DeBruijnIndex(i + 1), origin);
                    e = self.store_checked(CorePrismExpr::FnDestruct(e, v), origin);
                }
                let g = self.store_checked(CorePrismExpr::GrammarValue(grammar), origin);
                let e = self.store_checked(CorePrismExpr::FnDestruct(e, g), origin);

                CorePrismExpr::FnConstruct(e)
            }
            ParsedPrismExpr::ShiftTo {
                expr,
                captured_env,
                adapt_env_len,
                grammar,
            } => {
                let old_names = jump_labels[&(grammar as *const _)];

                let mut names = NamesEnv::default();
                for (name, entry) in old_names.into_iter().collect::<Vec<_>>().into_iter().rev() {
                    let NamesEntry::FromEnv(i) = entry else {
                        //TODO this is probably possible to hit but niche
                        unreachable!()
                    };

                    names = names.insert(
                        name,
                        NamesEntry::FromGrammarEnv {
                            adapt_env_len,
                            prev_env_len: i,
                        },
                        self.allocs,
                    );
                }

                for (name, value) in captured_env
                    .into_iter()
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                {
                    names =
                        names.insert(name, NamesEntry::FromParsed(value, env.names), self.allocs);
                }

                let env = NamedEnv::<'arn, 'grm> {
                    env_len: env.env_len,
                    names,
                    //TODO should these be preserved?
                    hygienic_names: Default::default(),
                };

                return self.parsed_to_checked_with_env(expr, env, jump_labels);
            }
        };
        self.store_checked(e, origin)
    }
}
