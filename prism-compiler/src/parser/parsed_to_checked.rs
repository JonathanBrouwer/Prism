use crate::lang::error::TypeError;
use crate::lang::type_check::{NamedEnv, NamesEntry};
use crate::lang::{CheckedIndex, CheckedPrismExpr, PrismEnv, ValueOrigin};
use crate::parser::{ParsedIndex, ParsedPrismExpr};
use prism_parser::core::input::Input;

impl<'arn, 'grm: 'arn> PrismEnv<'arn, 'grm> {
    pub fn parsed_to_checked(&mut self, i: ParsedIndex) -> CheckedIndex {
        self.parsed_to_checked_with_env(i, &NamedEnv::default())
    }

    pub fn parsed_to_checked_with_env(
        &mut self,
        i: ParsedIndex,
        env: &NamedEnv<'arn, 'grm>,
    ) -> CheckedIndex {
        let e = match self.parsed_values[*i] {
            ParsedPrismExpr::Free => CheckedPrismExpr::Free,
            ParsedPrismExpr::Type => CheckedPrismExpr::Type,
            ParsedPrismExpr::Let(n, v, b) => CheckedPrismExpr::Let(
                self.parsed_to_checked_with_env(v, env),
                self.parsed_to_checked_with_env(b, &env.insert_name(n, self.input)),
            ),
            ParsedPrismExpr::FnType(n, a, b) => CheckedPrismExpr::FnType(
                self.parsed_to_checked_with_env(a, env),
                self.parsed_to_checked_with_env(b, &env.insert_name(n, self.input)),
            ),
            ParsedPrismExpr::FnConstruct(n, b) => CheckedPrismExpr::FnConstruct(
                self.parsed_to_checked_with_env(b, &env.insert_name(n, self.input)),
            ),
            ParsedPrismExpr::FnDestruct(f, a) => CheckedPrismExpr::FnDestruct(
                self.parsed_to_checked_with_env(f, env),
                self.parsed_to_checked_with_env(a, env),
            ),
            ParsedPrismExpr::TypeAssert(v, t) => CheckedPrismExpr::TypeAssert(
                self.parsed_to_checked_with_env(v, env),
                self.parsed_to_checked_with_env(t, env),
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
            ParsedPrismExpr::ShiftLabel(b, guid) => {
                return self.parsed_to_checked_with_env(b, &env.insert_shift_label(guid));
            }
            ParsedPrismExpr::ShiftTo(b, guid, captured_env) => {
                let env = env.shift_to_label(guid, captured_env, self);
                return self.parsed_to_checked_with_env(b, &env);
            }
            ParsedPrismExpr::ParserValue(v) => CheckedPrismExpr::ParserValue(v),
            ParsedPrismExpr::ParsedType => CheckedPrismExpr::ParsedType,
        };
        self.store_checked(e, ValueOrigin::SourceCode(self.parsed_spans[*i]))
    }
}
