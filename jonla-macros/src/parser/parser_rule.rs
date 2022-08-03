use crate::grammar::{RuleAction, RuleBody};
use crate::parser::action_result::ActionResult;
use crate::parser::core::error::ParseError;
use crate::parser::core::parser::Parser;
use crate::parser::core::presult::PResult;
use crate::parser::core::primitives::{repeat_delim, single};
use crate::parser::core::stream::Stream;
use crate::parser::parser_state::{parser_cache_recurse, ParserState};
use itertools::Itertools;
use std::collections::HashMap;

pub type PR<'grm> = (HashMap<&'grm str, ActionResult<'grm>>, ActionResult<'grm>);

pub fn parser_rule<'grm, S: Stream<I = char>, E: ParseError + Clone>(
    rules: &'grm HashMap<&'grm str, RuleBody<'grm>>,
    rule: &'grm str,
) -> impl Parser<char, PR<'grm>, S, E, ParserState<'grm, PResult<PR<'grm>, E, S>>> {
    move |stream: S,
          state: &mut ParserState<'grm, PResult<PR<'grm>, E, S>>|
          -> PResult<PR<'grm>, E, S> {
        //TODO wrap in cache_recurse
        parser_cache_recurse(
        &parser_expr(rules, &rules.get(rule).unwrap()),
            rule
        ).parse(stream, state)
    }
}

fn parser_expr<'grm, S: Stream<I = char>, E: ParseError + Clone>(
    rules: &'grm HashMap<&'grm str, RuleBody<'grm>>,
    expr: &'grm RuleBody<'grm>,
) -> impl Parser<char, PR<'grm>, S, E, ParserState<'grm, PResult<PR<'grm>, E, S>>> {
    move |stream: S,
          state: &mut ParserState<'grm, PResult<PR<'grm>, E, S>>|
          -> PResult<PR<'grm>, E, S> {
        match expr {
            RuleBody::Rule(rule) => parser_rule(rules, rule)
                .parse(stream, state)
                .map(|(_, v)| (HashMap::new(), v)),
            RuleBody::CharClass(cc) => single(|c| cc.contains(*c))
                .parse(stream, state)
                .map(|(span, _)| (HashMap::new(), ActionResult::Value(span))),
            RuleBody::Literal(literal) => {
                let mut res = PResult::new_ok((), stream);
                for char in literal.chars() {
                    res = res.merge_seq(&single(|c| *c == char), state).map(|_| ());
                }
                let span = stream.span_to(res.get_stream());
                res.map(|_| (HashMap::new(), ActionResult::Value(span)))
            }
            RuleBody::Repeat {
                expr,
                min,
                max,
                delim,
            } => {
                repeat_delim(
                    parser_expr(rules, expr),
                    parser_expr(rules, delim),
                    *min as usize,
                    max.map(|max| max as usize)
                )
                    .parse(stream, state)
                    .map(|list| (HashMap::new(), ActionResult::List(list.into_iter().map(|pr| pr.1).collect_vec())))
            }
            RuleBody::Sequence(subs) => {
                let mut res = PResult::new_ok(HashMap::new(), stream);
                for sub in subs {
                    res = res.merge_seq(&parser_expr(rules, sub), state).map(|(mut l, r)| {
                        l.extend(r.0);
                        l
                    });
                }
                res.map(|map| (map, ActionResult::Error))
            }
            RuleBody::Choice(subs) => {
                let mut res: PResult<PR, E, S> = parser_expr(rules, &subs[0]).parse(stream, state);
                for sub in &subs[1..] {
                    res = res.merge_choice(&parser_expr(rules, &sub), stream, state);
                }
                res.map(|(_, v)| (HashMap::new(), v))
            }
            RuleBody::NameBind(name, sub) => {
                let res = parser_expr(rules, sub).parse(stream, state);
                res.map(|mut res| {
                    res.0.insert(name, res.1.clone());
                    res
                })
            }
            RuleBody::Action(sub, action) => {
                let res = parser_expr(rules, sub).parse(stream, state);
                res.map(|mut res| {
                    res.1 = apply_action(action, &res.0);
                    res
                })
            }
            RuleBody::SliceInput(sub) => {
                let res = parser_expr(rules, sub).parse(stream, state);
                let span = stream.span_to(res.get_stream());
                res.map(|_| (HashMap::new(), ActionResult::Value(span)))
            }
            RuleBody::Error(sub, _err_label) => {
                let res = parser_expr(rules, sub).parse(stream, state);
                //TODO implement
                // res.map_errs(|mut err| {
                //     err.labels = vec![ParseErrorLabel::Error(err_label)];
                //     err.start = Some(pos);
                //     err
                // })
                res
            }
        }
    }
}

fn apply_action<'grm>(
    rule: &RuleAction<'grm>,
    map: &HashMap<&str, ActionResult<'grm>>,
) -> ActionResult<'grm> {
    match rule {
        RuleAction::Name(name) => {
            if let Some(v) = map.get(name) {
                v.clone()
            } else {
                ActionResult::Error
            }
        }
        RuleAction::InputLiteral(lit) => ActionResult::Literal(lit),
        RuleAction::Construct(name, args) => {
            let args_vals = args.iter().map(|a| apply_action(a, map)).collect_vec();
            ActionResult::Construct(name, args_vals)
        }
    }
}

// pub fn parser_full_input<T: Clone>(
//     input: &str,
//     sub: impl Fn(&mut ParserState<'grm, 'src, CT>, usize) -> ParseResult<'grm, T>,
// ) -> PResult<_, _, _> {
//     let res = sub(self, 0);
//     match res.inner {
//         Ok(ok) if res.pos() == self.input.len() => ParseResult::from_ok(ok),
//         Ok(ok) => ok
//             .best_error
//             .map(ParseResult::from_err)
//             .unwrap_or(ParseResult::new_err(ok.pos, vec![RemainingInputNotParsed])),
//         Err(err) => ParseResult::from_err(err),
//     }
// }