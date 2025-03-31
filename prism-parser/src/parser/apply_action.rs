use crate::core::allocs::alloc_extend;
use crate::core::pos::Pos;
use crate::core::span::Span;
use crate::core::state::ParserState;
use crate::error::ParseError;
use crate::error::error_printer::ErrorLabel;
use crate::grammar::identifier::Identifier;
use crate::grammar::rule_action::RuleAction;
use crate::parsable::parsed::{ArcExt, Parsed};
use crate::parsable::void::Void;
use crate::parser::VarMap;
use crate::parser::placeholder_store::ParsedPlaceholder;
use std::collections::HashMap;
use std::iter;
use std::sync::Arc;

impl<Env, E: ParseError<L = ErrorLabel>> ParserState<Env, E> {
    pub fn pre_apply_action(
        &mut self,
        rule: &RuleAction,
        penv: &mut Env,
        pos: Pos,

        placeholder: ParsedPlaceholder,
        eval_ctx: &Parsed,
        eval_ctxs: &mut HashMap<String, (Parsed, ParsedPlaceholder)>,
    ) {
        match rule {
            RuleAction::Name(n) => {
                let n = n.as_str(&self.input);
                if eval_ctxs.contains_key(n) {
                    // Ctx is ambiguous
                    //TODO if both ctxs are identical, we can continue
                    eval_ctxs.insert(n.to_string(), (Arc::new(Void).to_parsed(), placeholder));
                } else {
                    eval_ctxs.insert(n.to_string(), (eval_ctx.clone(), placeholder));
                }
            }
            RuleAction::Construct {
                ns: namespace,
                name: constructor,
                args,
            } => {
                let namespace = namespace.as_str(&self.input);

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
                    *constructor,
                    *ns,
                    placeholders.clone(),
                );

                // Create envs for args
                let arg_envs =
                    (ns.create_eval_ctx)(*constructor, &eval_ctx, &placeholders, &self.input, penv);

                // Recurse on args
                for ((arg, env), placeholder) in args
                    .iter()
                    .zip(
                        arg_envs
                            .iter()
                            .cloned()
                            .chain(iter::repeat(Arc::new(Void).to_parsed())),
                    )
                    .zip(&placeholders)
                {
                    self.pre_apply_action(arg, penv, pos, *placeholder, &env, eval_ctxs);
                }
            }
            RuleAction::InputLiteral(lit) => {
                let parsed = Arc::new(*lit).to_parsed();
                self.placeholders.place_into_empty(
                    placeholder,
                    parsed,
                    pos.span_to(pos),
                    &self.input,
                    penv,
                );
            }
            RuleAction::Value { .. } => {
                //TODO
            }
        }
    }

    pub fn apply_action(
        &self,
        rule: &RuleAction,
        span: Span,
        vars: &VarMap,
        penv: &mut Env,
    ) -> Parsed {
        match rule {
            RuleAction::Name(name) => {
                if let Some(ar) = vars.get_ident(*name, &self.input) {
                    ar.clone()
                } else {
                    panic!("Name '{}' not in context", name.as_str(&self.input))
                }
            }
            RuleAction::InputLiteral(lit) => Arc::new(*lit).to_parsed(),
            RuleAction::Construct { ns, name, args } => {
                let ns = ns.as_str(&self.input);

                let ns = self
                    .parsables
                    .get(ns)
                    .unwrap_or_else(|| panic!("Namespace '{ns}' exists"));
                let args_vals =
                    alloc_extend(args.iter().map(|a| self.apply_action(a, span, vars, penv)));
                (ns.from_construct)(span, *name, &args_vals, &self.input, penv)
            }
            RuleAction::Value { ns, value } => {
                let ns = ns.as_str(&self.input);

                let ns = self
                    .parsables
                    .get(ns)
                    .unwrap_or_else(|| panic!("Namespace '{ns}' exists"));
                (ns.from_construct)(
                    span,
                    Identifier::from_const("EnvCapture"),
                    &[value.clone(), Arc::new(vars.clone()).to_parsed()],
                    &self.input,
                    penv,
                )
            }
        }
    }
}
