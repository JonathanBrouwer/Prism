use crate::core::context::{PCache, ParserContext};
use crate::error::error_printer::ErrorLabel;
use crate::error::error_printer::ErrorLabel::Debug;
use crate::error::ParseError;
use crate::core::parser::Parser;
use crate::core::presult::PResult;
use crate::core::presult::PResult::{PErr, POk};
use crate::core::span::Span;
use crate::core::stream::StringStream;

pub fn empty<'b, 'grm: 'b, E: ParseError>() -> impl Parser<'b, 'grm, (), E> {
    move |pos: StringStream<'grm>,
          _: &mut PCache<'b, 'grm, E>,
          _: &ParserContext<'b, 'grm>|
          -> PResult<'grm, (), E> { PResult::new_ok((), pos) }
}

pub fn single<'b, 'grm: 'b, E: ParseError>(
    f: impl Fn(&char) -> bool,
) -> impl Parser<'b, 'grm, (Span, char), E> {
    move |pos: StringStream<'grm>,
          _: &mut PCache<'b, 'grm, E>,
          _: &ParserContext<'b, 'grm>|
          -> PResult<'grm, (Span, char), E> {
        match pos.next() {
            // We can parse the character
            (pos_new, Some((span, e))) if f(&e) => PResult::new_ok((span, e), pos_new),
            // // We cannot parse the character but we can recover
            // (_, Some((_, e))) if (ctx.recovery_disabled.is_none() || ctx.recovery_disabled.unwrap() == pos.pos()) && ctx.recovery_points.contains_key(&pos.pos()) => {
            //     let end = *ctx.recovery_points.get(&pos.pos()).unwrap();
            //     PResult::new_ok(
            //         (
            //             Span {
            //                 start: pos.pos(),
            //                 end,
            //             },
            //             e,
            //         ),
            //         pos.with_pos(end),
            //     )
            // }
            // Error
            (pos_new, _) => PResult::new_err(E::new(pos.span_to(pos_new)), pos),
        }
    }
}

pub fn seq2<'b, 'grm: 'b, 'a, O1, O2, E: ParseError>(
    p1: &'a impl Parser<'b, 'grm, O1, E>,
    p2: &'a impl Parser<'b, 'grm, O2, E>,
) -> impl Parser<'b, 'grm, (O1, O2), E> + 'a {
    move |stream: StringStream<'grm>,
          cache: &mut PCache<'b, 'grm, E>,
          context: &ParserContext<'b, 'grm>|
          -> PResult<'grm, (O1, O2), E> {
        let res1 = p1.parse(stream, cache, context);
        let stream = res1.get_stream();
        res1.merge_seq(p2.parse(stream, cache, context))
    }
}

pub fn choice2<'b, 'grm: 'b, 'a, O, E: ParseError>(
    p1: &'a impl Parser<'b, 'grm, O, E>,
    p2: &'a impl Parser<'b, 'grm, O, E>,
) -> impl Parser<'b, 'grm, O, E> + 'a {
    move |stream: StringStream<'grm>,
          cache: &mut PCache<'b, 'grm, E>,
          context: &ParserContext<'b, 'grm>|
          -> PResult<'grm, O, E> {
        p1.parse(stream, cache, context)
            .merge_choice(p2.parse(stream, cache, context))
    }
}

pub fn repeat_delim<
    'b,
    'grm: 'b,
    OP,
    OD,
    E: ParseError<L = ErrorLabel<'grm>>,
    P: Parser<'b, 'grm, OP, E>,
    D: Parser<'b, 'grm, OD, E>,
>(
    item: P,
    delimiter: D,
    min: usize,
    max: Option<usize>,
) -> impl Parser<'b, 'grm, Vec<OP>, E> {
    move |stream: StringStream<'grm>,
          cache: &mut PCache<'b, 'grm, E>,
          context: &ParserContext<'b, 'grm>|
          -> PResult<'grm, Vec<OP>, E> {
        let mut last_res: PResult<'grm, Vec<OP>, E> = PResult::new_ok(vec![], stream);

        for i in 0..max.unwrap_or(usize::MAX) {
            let pos = last_res.get_stream();
            let part = if i == 0 {
                item.parse(pos, cache, context)
            } else {
                seq2(&delimiter, &item)
                    .parse(pos, cache, context)
                    .map(|x| x.1)
            };
            let should_continue = part.is_ok();

            if i < min {
                last_res = last_res.merge_seq(part).map(|(mut vec, item)| {
                    vec.push(item);
                    vec
                });
            } else {
                last_res = last_res.merge_seq_opt(part).map(|(mut vec, item)| {
                    if let Some(item) = item {
                        vec.push(item);
                    }
                    vec
                });
            };

            if !should_continue {
                break;
            };

            // If the result is OK and the last pos has not changed, we got into an infinite loop
            // We break out with an infinite loop error
            // The i != 0 check is to make sure to take the delim into account
            if i != 0 && pos.pos() == last_res.get_stream().pos() {
                let span = Span::new(pos.pos(), pos.pos());
                let mut e = E::new(span);
                e.add_label_explicit(Debug(span, "INFLOOP"));
                return PResult::new_err(e, pos);
            }
        }

        last_res
    }
}

pub fn end<'b, 'grm: 'b, E: ParseError>() -> impl Parser<'b, 'grm, (), E> {
    move |stream: StringStream<'grm>,
          _: &mut PCache<'b, 'grm, E>,
          _: &ParserContext<'b, 'grm>|
          -> PResult<'grm, (), E> {
        match stream.next() {
            (s, Some(_)) => PResult::new_err(E::new(stream.span_to(s)), stream),
            (s, None) => PResult::new_ok((), s),
        }
    }
}

pub fn positive_lookahead<'b, 'grm: 'b, O, E: ParseError>(
    p: &impl Parser<'b, 'grm, O, E>,
) -> impl Parser<'b, 'grm, O, E> + '_ {
    move |stream: StringStream<'grm>,
          cache: &mut PCache<'b, 'grm, E>,
          context: &ParserContext<'b, 'grm>|
          -> PResult<'grm, O, E> {
        match p.parse(stream, cache, context) {
            POk(o, _, err) => POk(o, stream, err),
            PErr(e, s) => PErr(e, s),
        }
    }
}

pub fn negative_lookahead<'b, 'grm: 'b, O, E: ParseError>(
    p: &impl Parser<'b, 'grm, O, E>,
) -> impl Parser<'b, 'grm, (), E> + '_ {
    move |stream: StringStream<'grm>,
          cache: &mut PCache<'b, 'grm, E>,
          context: &ParserContext<'b, 'grm>|
          -> PResult<'grm, (), E> {
        match p.parse(stream, cache, context) {
            POk(_, _, _) => PResult::new_err(E::new(stream.span_to(stream)), stream),
            PErr(_, _) => PResult::new_ok((), stream),
        }
    }
}