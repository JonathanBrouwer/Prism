use crate::lang::env::DbEnv;
use crate::lang::error::TypeError;
use crate::lang::{CoreIndex, CorePrismExpr, PrismEnv, ValueOrigin};
use crate::parser::named_env::{NamedEnv, NamesEntry, NamesEnv};
use crate::parser::parse_expr::EnvWrapper;
use crate::parser::{ParsedIndex, ParsedPrismExpr};
use prism_parser::core::span::Span;
use prism_parser::grammar::grammar_file::GrammarFile;
use prism_parser::parsable::ParseResult;
use prism_parser::parsable::guid::Guid;
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
            ParsedPrismExpr::ShiftTo {
                expr: b,
                vars: captured_env,
            } => {
                let mut names = env.names;
                // let mut names = jump_labels[&guid];

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

                return self.parsed_to_checked_with_env(b, env, jump_labels);
            }
            ParsedPrismExpr::GrammarValue(grammar) => {
                env.insert_shift_label(grammar, jump_labels);
                CorePrismExpr::GrammarValue(grammar)
            }
            ParsedPrismExpr::GrammarType => CorePrismExpr::GrammarType,
        };
        self.store_checked(e, ValueOrigin::SourceCode(self.parsed_spans[*i]))
    }
}
