use crate::grammar::{GrammarFile, RuleAction, RuleExpr};
use crate::parser::actual::action_result::ActionResult;
use crate::parser::actual::error_printer::ErrorLabel;
use crate::parser::actual::parser_layout::parser_with_layout;
use crate::parser::core::error::ParseError;
use crate::parser::core::parser::Parser;
use crate::parser::core::presult::PResult;
use crate::parser::core::primitives::{
    negative_lookahead, positive_lookahead, repeat_delim, single,
};

use crate::parser::actual::parser_rule::{parser_rule, ParserContext, PR, PState};
use crate::parser::actual::parser_rule_body::parser_body_cache_recurse;
use crate::parser::core::stream::StringStream;
use std::collections::HashMap;
use std::rc::Rc;

pub fn parser_expr<'a, 'b: 'a, 'grm: 'b, E: ParseError<L = ErrorLabel<'grm>> + Clone>(
    rules: &'grm GrammarFile,
    expr: &'grm RuleExpr,
    context: &'a ParserContext<'grm>,
) -> impl Parser<'grm, PR<'grm>, E, PState<'b, 'grm, E>> + 'a {
    move |stream: StringStream<'grm>,
          state: &mut PState<'b, 'grm, E>|
          -> PResult<'grm, PR<'grm>, E> {
        match expr {
            RuleExpr::Rule(rule) => parser_rule(
                rules,
                rule,
                &ParserContext {
                    layout_disabled: false,
                    prec_climb_this: None,
                    prec_climb_next: None,
                },
            )
            .parse(stream, state)
            .map(|(_, v)| (HashMap::new(), v)),
            RuleExpr::CharClass(cc) => {
                parser_with_layout(rules, &single(|c| cc.contains(*c)), context)
                    .parse(stream, state)
                    .map(|(span, _)| (HashMap::new(), Rc::new(ActionResult::Value(span))))
            }
            RuleExpr::Literal(literal) => {
                //First construct the literal parser
                let parser_literal =
                    move |stream: StringStream<'grm>,
                          state: &mut PState<'b, 'grm, E>|
                          -> PResult<'grm, PR<'grm>, E> {
                        let mut res = PResult::new_ok((), stream);
                        for char in literal.chars() {
                            res = res
                                .merge_seq_parser(&single(|c| *c == char), state)
                                .map(|_| ());
                        }
                        let span = stream.span_to(res.get_stream());
                        let mut res =
                            res.map(|_| (HashMap::new(), Rc::new(ActionResult::Value(span))));
                        res.add_label_implicit(ErrorLabel::Literal(
                            stream.span_to(res.get_stream().next().0),
                            literal,
                        ));
                        res
                    };
                //Next, allow there to be layout before the literal
                let res = parser_with_layout(rules, &parser_literal, context).parse(stream, state);
                res
            }
            RuleExpr::Repeat {
                expr,
                min,
                max,
                delim,
            } => repeat_delim(
                parser_expr(rules, expr, context),
                parser_expr(rules, delim, context),
                *min as usize,
                max.map(|max| max as usize),
            )
            .parse(stream, state)
            .map(|list| {
                (
                    HashMap::new(),
                    Rc::new(ActionResult::List(
                        list.into_iter().map(|pr| pr.1).collect(),
                    )),
                )
            }),
            RuleExpr::Sequence(subs) => {
                let mut res = PResult::new_ok(HashMap::new(), stream);
                for sub in subs {
                    res = res
                        .merge_seq_parser(&parser_expr(rules, sub, context), state)
                        .map(|(mut l, r)| {
                            l.extend(r.0);
                            l
                        });
                    if res.is_err() {
                        break;
                    }
                }
                res.map(|map| (map, Rc::new(ActionResult::Void("sequence"))))
            }
            RuleExpr::Choice(subs) => {
                let mut res: PResult<'grm, PR, E> =
                    PResult::PErr(E::new(stream.span_to(stream)), stream);
                for sub in subs {
                    res = res.merge_choice_parser(&parser_expr(rules, sub, context), stream, state);
                    if res.is_ok() {
                        break;
                    }
                }
                res
            }
            RuleExpr::NameBind(name, sub) => {
                let res = parser_expr(rules, sub, context).parse(stream, state);
                res.map(|mut res| {
                    if let ActionResult::Void(v) = *res.1 {
                        panic!("Tried to bind a void value '{v}' with name '{name}'")
                    }
                    res.0.insert(name, res.1.clone());
                    res
                })
            }
            RuleExpr::Action(sub, action) => {
                let res = parser_expr(rules, sub, context).parse(stream, state);
                res.map(|mut res| {
                    res.1 = apply_action(action, &res.0);
                    res
                })
            }
            RuleExpr::SliceInput(sub) => {
                let res = parser_expr(rules, sub, context).parse(stream, state);
                let span = stream.span_to(res.get_stream());
                res.map(|_| (HashMap::new(), Rc::new(ActionResult::Value(span))))
            }
            RuleExpr::AtThis => parser_body_cache_recurse(
                rules,
                *context.prec_climb_this.unwrap(),
                // Reset this/next as they shouldn't matter from now on
                &ParserContext {
                    prec_climb_this: None,
                    prec_climb_next: None,
                    ..*context
                },
            )
            .parse(stream, state)
            .map(|(_, v)| (HashMap::new(), v)),
            RuleExpr::AtNext => parser_body_cache_recurse(
                rules,
                *context.prec_climb_next.unwrap(),
                // Reset this/next as they shouldn't matter from now on
                &ParserContext {
                    prec_climb_this: None,
                    prec_climb_next: None,
                    ..*context
                },
            )
            .parse(stream, state)
            .map(|(_, v)| (HashMap::new(), v)),
            RuleExpr::PosLookahead(sub) => positive_lookahead(&parser_expr(rules, sub, context))
                .parse(stream, state)
                .map(|r| (HashMap::new(), r.1)),
            RuleExpr::NegLookahead(sub) => negative_lookahead(&parser_expr(rules, sub, context))
                .parse(stream, state)
                .map(|_| {
                    (
                        HashMap::new(),
                        Rc::new(ActionResult::Void("negative lookahead")),
                    )
                }),
        }
    }
}

fn apply_action<'grm>(
    rule: &'grm RuleAction,
    map: &HashMap<&str, Rc<ActionResult<'grm>>>,
) -> Rc<ActionResult<'grm>> {
    match rule {
        RuleAction::Name(name) => {
            if let Some(v) = map.get(&name[..]) {
                v.clone()
            } else {
                panic!("Name '{name}' not in context")
            }
        }
        RuleAction::InputLiteral(lit) => Rc::new(ActionResult::Literal(lit)),
        RuleAction::Construct(name, args) => {
            let args_vals = args.iter().map(|a| apply_action(a, map)).collect();
            Rc::new(ActionResult::Construct(name, args_vals))
        }
        RuleAction::Cons(h, t) => {
            let mut res = Vec::new();
            res.push(apply_action(h, map));
            res.extend_from_slice(match &*apply_action(t, map) {
                ActionResult::List(v) => &v[..],
                x => unreachable!("{:?} is not a list", x),
            });

            Rc::new(ActionResult::List(res))
        }
        RuleAction::Nil() => Rc::new(ActionResult::List(Vec::new())),
    }
}
