use crate::core::input::Input;
use crate::core::span::Span;
use crate::core::state::ParserState;
use crate::error::ParseError;
use crate::error::error_printer::ErrorLabel;
use crate::grammar::rule_action::RuleAction;
use crate::parsable::ParseResult;
use crate::parsable::env_capture::EnvCapture;
use crate::parsable::parsed::Parsed;
use crate::parsable::void::Void;
use crate::parser::VarMap;
use crate::parser::placeholder_store::ParsedPlaceholder;
use std::collections::HashMap;
use std::iter;

impl<'arn, 'grm: 'arn, Env, E: ParseError<L = ErrorLabel<'grm>>> ParserState<'arn, 'grm, Env, E> {
    pub fn pre_apply_action(
        &mut self,
        rule: &RuleAction<'arn, 'grm>,
        penv: &mut Env,

        placeholder: ParsedPlaceholder,
        eval_ctx: Parsed<'arn, 'grm>,
        eval_ctxs: &mut HashMap<&'grm str, (Parsed<'arn, 'grm>, ParsedPlaceholder)>,
    ) {
        match rule {
            RuleAction::Name(n) => {
                if eval_ctxs.contains_key(n) {
                    // Ctx is ambiguous
                    //TODO if both ctxs are identical, we can continue
                    eval_ctxs.insert(n, (Void.to_parsed(), placeholder));
                } else {
                    eval_ctxs.insert(n, (eval_ctx, placeholder));
                }
            }
            RuleAction::Construct {
                ns: namespace,
                name: constructor,
                args: args,
            } => {
                // Get placeholders for args
                let mut placeholders = Vec::with_capacity(args.len());
                for _arg in args.iter() {
                    placeholders.push(self.placeholders.push_empty());
                }

                // Store construct info
                let ns = self
                    .parsables
                    .get(namespace)
                    .unwrap_or_else(|| panic!("Namespace '{namespace}' exists"));
                self.placeholders.place_construct_info(
                    placeholder,
                    constructor,
                    *ns,
                    placeholders.clone(),
                );

                // Create envs for args
                let arg_envs = (ns.create_eval_ctx)(
                    constructor,
                    eval_ctx,
                    &placeholders,
                    self.alloc,
                    self.input,
                    penv,
                );

                // Recurse on args
                for ((arg, env), placeholder) in args
                    .iter()
                    .zip(
                        arg_envs
                            .iter()
                            .copied()
                            .chain(iter::repeat(Void.to_parsed())),
                    )
                    .zip(&placeholders)
                {
                    self.pre_apply_action(arg, penv, *placeholder, env, eval_ctxs);
                }
            }
            RuleAction::InputLiteral(lit) => {
                let parsed = self.alloc.alloc(Input::Literal(*lit)).to_parsed();
                self.placeholders.place_into_empty(
                    placeholder,
                    parsed,
                    self.alloc,
                    self.input,
                    penv,
                );
            }
            RuleAction::Value(_) => {
                //TODO
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
                    ar
                } else {
                    panic!("Name '{name}' not in context")
                }
            }
            RuleAction::InputLiteral(lit) => self.alloc.alloc(Input::Literal(*lit)).to_parsed(),
            RuleAction::Construct {
                ns: namespace,
                name: name,
                args: args,
            } => {
                let ns = self
                    .parsables
                    .get(namespace)
                    .unwrap_or_else(|| panic!("Namespace '{namespace}' exists"));
                let args_vals = self
                    .alloc
                    .alloc_extend(args.iter().map(|a| self.apply_action(a, span, vars, penv)));
                (ns.from_construct)(span, name, args_vals, self.alloc, self.input, penv)
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
