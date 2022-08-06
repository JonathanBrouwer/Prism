use crate::grammar::{RuleAction, RuleAnnotation, RuleBody, RuleExpr};
use crate::parser::action_result::ActionResult;
use crate::parser::core::error::ParseError;
use crate::parser::core::parser::Parser;
use crate::parser::core::presult::PResult;
use crate::parser::core::primitives::{repeat_delim, single};
use crate::parser::core::stream::Stream;
use crate::parser::error_printer::ErrorLabel;
use crate::parser::parser_state::{parser_cache_recurse, ParserState};
use itertools::Itertools;
use std::collections::HashMap;
use std::rc::Rc;

pub type PR<'grm> = (
    HashMap<&'grm str, Rc<ActionResult<'grm>>>,
    Rc<ActionResult<'grm>>,
);

pub fn parser_rule<
    'a,
    'grm: 'a,
    S: Stream<I = char>,
    E: ParseError<L = ErrorLabel<'grm>> + Clone,
>(
    rules: &'a HashMap<&'grm str, Vec<RuleBody<'grm>>>,
    rule: &'grm str,
) -> impl Parser<char, PR<'grm>, S, E, ParserState<'grm, PResult<PR<'grm>, E, S>>> + 'a {
    move |stream: S, state: &mut ParserState<'grm, PResult<PR<'grm>, E, S>>| -> PResult<PR<'grm>, E, S> {
        let mut res = parser_cache_recurse(
            &parser_body::<'_, 'grm, S, E>(rules, rules.get(rule).as_ref().unwrap()),
            rule,
        )
        .parse(stream, state);
        res.add_label(ErrorLabel::Debug(stream.span_to(res.get_stream()), rule));
        res
    }
}

fn parser_body<
    'a,
    'grm: 'a,
    S: Stream<I = char>,
    E: ParseError<L = ErrorLabel<'grm>> + Clone,
>(
    rules: &'a HashMap<&'grm str, Vec<RuleBody<'grm>>>,
    body: &'a Vec<RuleBody<'grm>>,
) -> impl Parser<char, PR<'grm>, S, E, ParserState<'grm, PResult<PR<'grm>, E, S>>> + 'a {
    move |stream: S, state: &mut ParserState<'grm, PResult<PR<'grm>, E, S>>| -> PResult<PR<'grm>, E, S> {
        let mut res: PResult<PR, E, S> = PResult::PErr(E::new(stream.span_to(stream)), stream);
        for body in body {
            res = res.merge_choice_parser(&parser_body_annots(rules, &body.annotations, &body.expr), stream, state);
            if res.is_ok() {
                break;
            }
        }
        res
    }
}

fn parser_body_annots<
    'a,
    'grm: 'a,
    S: Stream<I = char>,
    E: ParseError<L = ErrorLabel<'grm>> + Clone,
>(
    rules: &'a HashMap<&'grm str, Vec<RuleBody<'grm>>>,
    annots: &'a [RuleAnnotation<'grm>],
    expr: &'a RuleExpr<'grm>,
) -> impl Parser<char, PR<'grm>, S, E, ParserState<'grm, PResult<PR<'grm>, E, S>>> + 'a {
    move |stream: S, state: &mut ParserState<'grm, PResult<PR<'grm>, E, S>>| -> PResult<PR<'grm>, E, S> {
        match annots.get(0) {
            Some(att) => {
                let sub = parser_body_annots(rules, &annots[1..], expr);
                match att {
                    RuleAnnotation::Error(err_label) => {
                        let mut res = sub.parse(stream, state);
                        res.add_label(ErrorLabel::Explicit(
                            stream.span_to(res.get_stream().next().0),
                            err_label,
                        ));
                        res
                    }
                    RuleAnnotation::NoLayout => {
                        parser_with_layout(rules, &move |stream: S, state: &mut ParserState<'grm, PResult<PR<'grm>, E, S>>| -> PResult<_, E, S> {
                            state.layout_disable();
                            let res = sub.parse(stream, state);
                            state.layout_enable();
                            res
                        }).parse(stream, state)
                    }
                }
            }
            None => {
                parser_expr(rules, expr).parse(stream, state)
            }
        }
    }
}

fn parser_expr<'a, 'grm: 'a, S: Stream<I = char>, E: ParseError<L = ErrorLabel<'grm>> + Clone>(
    rules: &'a HashMap<&'grm str, Vec<RuleBody<'grm>>>,
    expr: &'a RuleExpr<'grm>,
) -> impl Parser<char, PR<'grm>, S, E, ParserState<'grm, PResult<PR<'grm>, E, S>>> + 'a {
    move |stream: S,
          state: &mut ParserState<'grm, PResult<PR<'grm>, E, S>>|
          -> PResult<PR<'grm>, E, S> {
        match expr {
            RuleExpr::Rule(rule) => parser_rule(rules, rule)
                .parse(stream, state)
                .map(|(_, v)| (HashMap::new(), v)),
            RuleExpr::CharClass(cc) => parser_with_layout(rules, &single(|c| cc.contains(*c)))
                .parse(stream, state)
                .map(|(span, _)| (HashMap::new(), Rc::new(ActionResult::Value(span)))),
            RuleExpr::Literal(literal) => {
                //First construct the literal parser
                let parser_literal =
                    move |stream: S,
                          state: &mut ParserState<'grm, PResult<PR<'grm>, E, S>>|
                          -> PResult<PR<'grm>, E, S> {
                        let mut res = PResult::new_ok((), stream);
                        for char in literal.chars() {
                            res = res
                                .merge_seq_parser(&single(|c| *c == char), state)
                                .map(|_| ());
                        }
                        let span = stream.span_to(res.get_stream());
                        let mut res =
                            res.map(|_| (HashMap::new(), Rc::new(ActionResult::Value(span))));
                        res.add_label(ErrorLabel::Literal(
                            stream.span_to(res.get_stream().next().0),
                            literal,
                        ));
                        res
                    };
                //Next, allow there to be layout before the literal
                let res = parser_with_layout(rules, &parser_literal).parse(stream, state);
                res
            }
            RuleExpr::Repeat {
                expr,
                min,
                max,
                delim,
            } => repeat_delim(
                parser_expr(rules, expr),
                parser_expr(rules, delim),
                *min as usize,
                max.map(|max| max as usize),
            )
            .parse(stream, state)
            .map(|list| {
                (
                    HashMap::new(),
                    Rc::new(ActionResult::List(
                        list.into_iter().map(|pr| pr.1).collect_vec(),
                    )),
                )
            }),
            RuleExpr::Sequence(subs) => {
                let mut res = PResult::new_ok(HashMap::new(), stream);
                for sub in subs {
                    res =
                        res.merge_seq_parser(&parser_expr(rules, sub), state)
                            .map(|(mut l, r)| {
                                l.extend(r.0);
                                l
                            });
                    if res.is_err() {
                        break;
                    }
                }
                res.map(|map| (map, Rc::new(ActionResult::Error("sequence"))))
            }
            RuleExpr::Choice(subs) => {
                let mut res: PResult<PR, E, S> = PResult::PErr(E::new(stream.span_to(stream)), stream);
                for sub in subs {
                    res = res.merge_choice_parser(&parser_expr(rules, &sub), stream, state);
                    if res.is_ok() {
                        break;
                    }
                }
                res
            }
            RuleExpr::NameBind(name, sub) => {
                let res = parser_expr(rules, sub).parse(stream, state);
                res.map(|mut res| {
                    res.0.insert(name, res.1.clone());
                    res
                })
            }
            RuleExpr::Action(sub, action) => {
                let res = parser_expr(rules, sub).parse(stream, state);
                res.map(|mut res| {
                    res.1 = apply_action(action, &res.0);
                    res
                })
            }
            RuleExpr::SliceInput(sub) => {
                let res = parser_expr(rules, sub).parse(stream, state);
                let span = stream.span_to(res.get_stream());
                res.map(|_| (HashMap::new(), Rc::new(ActionResult::Value(span))))
            }
        }
    }
}

fn parser_with_layout<
    'grm: 'a,
    'a,
    O,
    S: Stream<I = char>,
    E: ParseError<L = ErrorLabel<'grm>> + Clone,
>(
    rules: &'a HashMap<&'grm str, Vec<RuleBody<'grm>>>,
    sub: &'a impl Parser<char, O, S, E, ParserState<'grm, PResult<PR<'grm>, E, S>>>,
) -> impl Parser<char, O, S, E, ParserState<'grm, PResult<PR<'grm>, E, S>>> + 'a {
    move |pos: S, state: &mut ParserState<'grm, PResult<PR<'grm>, E, S>>| -> PResult<O, E, S> {
        if state.is_layout_disabled() || !rules.contains_key("layout") {
            return sub.parse(pos, state);
        }

        //Start attemping to parse layout
        let mut res = PResult::new_ok((), pos);
        loop {
            let (new_res, success) = res.merge_seq_opt_parser(sub, state);
            if success {
                return new_res.map(|(_, o)| o.unwrap());
            }

            state.layout_disable();
            res = new_res
                .merge_seq_parser(&parser_rule(rules, "layout"), state)
                .map(|_| ());
            state.layout_enable();
            if res.is_err() {
                return res.map(|_| unreachable!());
            }
        }
    }
}

fn apply_action<'grm>(
    rule: &RuleAction<'grm>,
    map: &HashMap<&str, Rc<ActionResult<'grm>>>,
) -> Rc<ActionResult<'grm>> {
    match rule {
        RuleAction::Name(name) => {
            if let Some(v) = map.get(name) {
                v.clone()
            } else {
                panic!("Name not in context")
            }
        }
        RuleAction::InputLiteral(lit) => Rc::new(ActionResult::Literal(lit)),
        RuleAction::Construct(name, args) => {
            let args_vals = args.iter().map(|a| apply_action(a, map)).collect_vec();
            Rc::new(ActionResult::Construct(name, args_vals))
        }
    }
}
