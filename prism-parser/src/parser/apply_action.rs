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
use crate::parser::placeholder_store::ParsedPlaceholder;
use crate::parser::var_map::VarMap;
use std::collections::HashMap;
use std::iter;

impl<'arn, 'grm: 'arn, Env, E: ParseError<L = ErrorLabel<'grm>>> ParserState<'arn, 'grm, Env, E> {
    pub fn pre_apply_action(
        &mut self,
        rule: &RuleAction<'arn, 'grm>,
        penv: &mut Env,

        placeholder: Option<ParsedPlaceholder>,
        eval_ctx: Parsed<'arn, 'grm>,
        eval_ctxs: &mut HashMap<&'grm str, (Parsed<'arn, 'grm>, Option<ParsedPlaceholder>)>,
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
            RuleAction::Construct(namespace, name, args) => {
                let ns = self
                    .parsables
                    .get(namespace)
                    .unwrap_or_else(|| panic!("Namespace '{namespace}' exists"));

                let mut placeholders = vec![];
                for _ in *args {
                    placeholders.push(self.placeholders.push(name, &placeholders));
                }

                let arg_envs = (ns.create_eval_ctx)(
                    name,
                    eval_ctx,
                    &placeholders,
                    self.alloc,
                    self.input,
                    penv,
                );
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
                    self.pre_apply_action(arg, penv, Some(*placeholder), env, eval_ctxs);
                }
            }
            RuleAction::InputLiteral(lit) => {
                if let Some(placeholder) = placeholder {
                    let parsed = self.alloc.alloc(Input::Literal(*lit)).to_parsed();
                    self.placeholders[placeholder] = parsed;
                }
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
