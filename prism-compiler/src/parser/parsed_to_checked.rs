use crate::lang::error::TypeError;
use crate::lang::{CheckedIndex, CheckedPrismExpr, PrismEnv, ValueOrigin};
use crate::parser::named_env::{NamedEnv, NamesEntry};
use crate::parser::{ParsedIndex, ParsedPrismExpr};
use prism_parser::core::input::Input;
use prism_parser::parsable::guid::Guid;
use rpds::HashTrieMap;
use std::collections::HashMap;

impl<'arn, 'grm: 'arn> PrismEnv<'arn, 'grm> {
    pub fn parsed_to_checked(&mut self, i: ParsedIndex) -> CheckedIndex {
        self.parsed_to_checked_with_env(i, &NamedEnv::default(), &mut HashMap::new())
    }

    pub fn parsed_to_checked_with_env(
        &mut self,
        i: ParsedIndex,
        env: &NamedEnv<'arn, 'grm>,
        jump_labels: &mut HashMap<Guid, HashTrieMap<&'arn str, NamesEntry<'arn, 'grm>>>,
    ) -> CheckedIndex {
        let e = match self.parsed_values[*i] {
            ParsedPrismExpr::Free => CheckedPrismExpr::Free,
            ParsedPrismExpr::Type => CheckedPrismExpr::Type,
            ParsedPrismExpr::Let(n, v, b) => CheckedPrismExpr::Let(
                self.parsed_to_checked_with_env(v, env, jump_labels),
                self.parsed_to_checked_with_env(b, &env.insert_name(n, self.input), jump_labels),
            ),
            ParsedPrismExpr::FnType(n, a, b) => CheckedPrismExpr::FnType(
                self.parsed_to_checked_with_env(a, env, jump_labels),
                self.parsed_to_checked_with_env(b, &env.insert_name(n, self.input), jump_labels),
            ),
            ParsedPrismExpr::FnConstruct(n, b) => CheckedPrismExpr::FnConstruct(
                self.parsed_to_checked_with_env(b, &env.insert_name(n, self.input), jump_labels),
            ),
            ParsedPrismExpr::FnDestruct(f, a) => CheckedPrismExpr::FnDestruct(
                self.parsed_to_checked_with_env(f, env, jump_labels),
                self.parsed_to_checked_with_env(a, env, jump_labels),
            ),
            ParsedPrismExpr::TypeAssert(v, t) => CheckedPrismExpr::TypeAssert(
                self.parsed_to_checked_with_env(v, env, jump_labels),
                self.parsed_to_checked_with_env(t, env, jump_labels),
            ),
            ParsedPrismExpr::Name(name) => {
                assert_ne!(name, "_");

                match env.resolve_name_use(name) {
                    Some(NamesEntry::FromEnv(prev_env_len)) => {
                        CheckedPrismExpr::DeBruijnIndex(env.len() - *prev_env_len - 1)
                    }
                    Some(NamesEntry::FromParsed(parsed, old_names)) => {
                        if let Some(&expr) = parsed.try_into_value::<ParsedIndex>() {
                            return self.parsed_to_checked_with_env(
                                expr,
                                &env.shift_back(old_names, self.input),
                                jump_labels,
                            );
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
            ParsedPrismExpr::ShiftTo(b, guid, captured_env) => {
                let env = env.shift_to_label(guid, captured_env, self, jump_labels);
                return self.parsed_to_checked_with_env(b, &env, jump_labels);
            }
            ParsedPrismExpr::GrammarValue(v, guid) => {
                env.insert_shift_label(guid, jump_labels);
                CheckedPrismExpr::GrammarValue(v)
            }
            ParsedPrismExpr::GrammarType => CheckedPrismExpr::GrammarType,
        };
        self.store_checked(e, ValueOrigin::SourceCode(self.parsed_spans[*i]))
    }
}
