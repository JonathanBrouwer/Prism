use crate::core::cache::PCache;
use crate::core::context::{Ignore, ParserContext, PR, Raw};
use crate::core::parser::Parser;
use crate::core::pos::Pos;
use crate::core::presult::PResult;
use crate::core::presult::PResult::{PErr, POk};
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::rule_action::{ActionResult, RuleAction};
use std::collections::HashMap;
use std::sync::Arc;

pub fn parse_with_recovery<'a, 'b: 'a, 'grm: 'b, O, E: ParseError<L = ErrorLabel<'grm>> + Clone>(
    sub: &'a impl Parser<'b, 'grm, O, E>,
    stream: Pos,
    cache: &mut PCache<'b, 'grm, E>,
    context: &ParserContext<'b, 'grm>,
) -> Result<O, Vec<E>> {
    let mut recovery_points: HashMap<Pos, Pos> = HashMap::new();
    let mut result_errors: Vec<E> = Vec::new();
    let mut err_state: Option<(Pos, Pos)> = None;

    loop {
        let context = ParserContext {
            recovery_points: Ignore(Arc::new(recovery_points.clone())),
            ..context.clone()
        };

        match sub.parse(stream, cache, &context) {
            POk(o, _, _, _, _) => {
                return if result_errors.is_empty() {
                    Ok(o)
                } else {
                    // Update last error
                    if let Some(last) = result_errors.last_mut() {
                        last.set_end(err_state.unwrap().1);
                    }
                    Err(result_errors)
                };
            }
            PErr(e, p) => {
                //If this is the first time we encounter *this* error, log it and retry
                if err_state.is_none() || err_state.unwrap().1 < p {
                    // Update last error
                    if let Some(last) = result_errors.last_mut() {
                        last.set_end(err_state.unwrap().1);
                    }

                    // Add new error
                    result_errors.push(e);
                    err_state = Some((p, p));
                } else if let Some((_err_state_start, err_state_end)) = &mut err_state {
                    //If the error now spans rest of file, we could not recover
                    if *err_state_end == Pos::end(cache.input) {
                        result_errors
                            .last_mut()
                            .unwrap()
                            .set_end(Pos::end(cache.input));
                        return Err(result_errors);
                    }

                    //Increase offset by one char and repeat
                    *err_state_end = err_state_end.next(cache.input).0;
                    debug_assert!(*err_state_end <= Pos::end(cache.input));
                } else {
                    unreachable!()
                }
                recovery_points.insert(err_state.unwrap().0, err_state.unwrap().1);
                recovery_points.insert(err_state.unwrap().1, err_state.unwrap().1);
                cache.clear();
            }
        }
    }
}

pub fn recovery_point<'a, 'b: 'a, 'grm: 'b, E: ParseError<L = ErrorLabel<'grm>>>(
    item: impl Parser<'b, 'grm, PR<'grm>, E> + 'a,
) -> impl Parser<'b, 'grm, PR<'grm>, E> + 'a {
    move |stream: Pos,
          cache: &mut PCache<'b, 'grm, E>,
          context: &ParserContext<'b, 'grm>|
          -> PResult<PR<'grm>, E> {
        // First try original parse
        match item.parse(
            stream,
            cache,
            &ParserContext {
                recovery_disabled: true,
                ..context.clone()
            },
        ) {
            r @ POk(_, _, _, _, _) => r,
            PErr(e, s) => {
                if let Some(to) = context.recovery_points.get(&s) {
                    POk(
                        //TODO recovery nicer
                        PR(HashMap::new(), Raw::Internal("Recovered")),
                        stream,
                        *to,
                        true,
                        Some((e, s)),
                    )
                } else {
                    PErr(e, s)
                }
            }
        }
    }
}
