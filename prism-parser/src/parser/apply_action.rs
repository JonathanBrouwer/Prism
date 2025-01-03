use crate::core::cache::Allocs;
use crate::core::input::Input;
use crate::core::span::Span;
use crate::core::state::ParserState;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::grammar::rule_action::RuleAction;
use crate::parsable::env_capture::EnvCapture;
use crate::parsable::parsed::Parsed;
use crate::parsable::parsed_mut::ParsedMut;
use crate::parsable::ParseResult;
use crate::parser::var_map::VarMap;
use std::collections::HashMap;

pub struct ActionEntry<'arn, 'grm: 'arn, Env> {
    arg: usize,
    arg_fn: fn(
        s: &mut ParsedMut<'arn, 'grm>,
        arg: usize,
        value: Parsed<'arn, 'grm>,
        allocs: Allocs<'arn>,
        src: &'grm str,
        env: &mut Env,
    ),
    value: ParsedMut<'arn, 'grm>,
}
impl<'arn, 'grm, Env> ActionEntry<'arn, 'grm, Env> {
    pub fn apply(
        &mut self,
        value: Parsed<'arn, 'grm>,
        allocs: Allocs<'arn>,
        src: &'grm str,
        env: &mut Env,
    ) {
        (self.arg_fn)(&mut self.value, self.arg, value, allocs, src, env)
    }
}

impl<'arn, 'grm: 'arn, Env, E: ParseError<L = ErrorLabel<'grm>>> ParserState<'arn, 'grm, Env, E> {
    pub fn apply_action_before(
        &self,
        bind: Option<ActionEntry<'arn, 'grm, Env>>,
        bind_map: &mut HashMap<&'grm str, ActionEntry<'arn, 'grm, Env>>,
        rule: &RuleAction<'arn, 'grm>,
        penv: &mut Env,
    ) {
        match rule {
            RuleAction::Name(name) => {
                if let Some(bind) = bind {
                    bind_map.insert(name, bind);
                }
            }
            RuleAction::Construct(namespace, name, args) => {
                let ns = self
                    .parsables
                    .get(namespace)
                    .unwrap_or_else(|| panic!("Namespace '{namespace}' exists"));
                let mut builder = (ns.build)(name, self.alloc, self.input, penv);
                for (i, arg) in args.iter().enumerate() {
                    self.apply_action_before(
                        Some(ActionEntry {
                            arg: i,
                            arg_fn: ns.arg,
                            value: builder.clone(),
                        }),
                        bind_map,
                        arg,
                        penv,
                    );
                }
            }
            RuleAction::InputLiteral(lit) => {
                let val = self.alloc.alloc(Input::Literal(*lit)).to_parsed();
                if let Some(mut bind) = bind {
                    bind.apply(val, self.alloc, self.input, penv);
                }
            }
            RuleAction::Value(_parsed) => {
                // Ignored
            }
        }
    }

    pub fn apply_action(
        &self,
        rule: &RuleAction<'arn, 'grm>,
        span: Span,
        vars: VarMap<'arn, 'grm>,
        penv: &mut Env,
    ) -> Parsed<'arn, 'grm> {
        match rule {
            RuleAction::Name(name) => {
                if let Some(ar) = vars.get(name) {
                    *ar
                } else {
                    panic!("Name '{name}' not in context")
                }
            }
            RuleAction::InputLiteral(lit) => self.alloc.alloc(Input::Literal(*lit)).to_parsed(),
            RuleAction::Construct(namespace, name, args) => {
                let ns = self
                    .parsables
                    .get(namespace)
                    .unwrap_or_else(|| panic!("Namespace '{namespace}' exists"));

                let mut builder = (ns.build)(name, self.alloc, self.input, penv);
                for (i, arg) in args.iter().enumerate() {
                    let arg = self.apply_action(arg, span, vars, penv);
                    (ns.arg)(&mut builder, i, arg, self.alloc, self.input, penv);
                }
                (ns.finish)(&mut builder, span, self.alloc, self.input, penv)
            }
            RuleAction::Value(parsed) => self
                .alloc
                .alloc(EnvCapture {
                    env: vars,
                    value: *parsed,
                })
                .to_parsed(),
        }
    }
}
