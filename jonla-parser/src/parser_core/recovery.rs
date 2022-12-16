use crate::parser_core::error::ParseError;
use crate::parser_core::parser::Parser;
use crate::parser_core::presult::PResult::{PErr, POk};
use crate::parser_core::stream::StringStream;
use crate::parser_sugar::error_printer::ErrorLabel;
use std::collections::HashMap;
use std::sync::Arc;
use crate::parser_sugar::parser_context::{Ignore, ParserContext, PState};

pub fn parse_with_recovery<'a, 'b: 'a, 'grm: 'b, O, E: ParseError<L = ErrorLabel<'grm>> + Clone>(
    sub: &'a impl Parser<'b, 'grm, O, E, PState<'b, 'grm, E>>,
    stream: StringStream<'grm>,
    cache: &mut PState<'b, 'grm, E>,
    context: &ParserContext<'b, 'grm>,
) -> Result<O, Vec<E>> {
    let mut recovery_points: HashMap<usize, usize> = HashMap::new();
    let mut result_errors: Vec<E> = Vec::new();
    let mut err_state: Option<(usize, usize)> = None;

    loop {
        let context = ParserContext {
            recovery_points: Ignore(Arc::new(recovery_points.clone())),
            ..context.clone()
        };

        match sub.parse(stream, cache, &context) {
            POk(o, _, _) => {
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
            PErr(e, s) => {
                //If this is the first time we encounter *this* error, log it and retry
                if err_state.is_none() || err_state.unwrap().1 < s.pos() {
                    // Update last error
                    if let Some(last) = result_errors.last_mut() {
                        last.set_end(err_state.unwrap().1);
                    }

                    // Add new error
                    result_errors.push(e);
                    err_state = Some((s.pos(), s.pos()));
                } else if let Some((_err_state_start, err_state_end)) = &mut err_state {
                    //If the error now spans rest of file, we could not recover
                    if *err_state_end == s.src().len() {
                        result_errors.last_mut().unwrap().set_end(s.src().len());
                        return Err(result_errors);
                    }

                    //Increase offset by one char and repeat
                    *err_state_end = stream.with_pos(*err_state_end).next().0.pos();
                    debug_assert!(*err_state_end <= s.src().len());
                } else {
                    unreachable!()
                }
                recovery_points.insert(err_state.unwrap().0, err_state.unwrap().1);
                cache.clear();
            }
        }
    }
}
