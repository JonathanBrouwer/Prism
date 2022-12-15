use std::collections::HashMap;
use std::sync::Arc;
use crate::parser_core::error::ParseError;
use crate::parser_core::parser::Parser;
use crate::parser_core::presult::PResult::{PErr, POk};
use crate::parser_core::stream::StringStream;
use crate::parser_sugar::error_printer::ErrorLabel;
use crate::parser_sugar::parser_rule::{Ignore, ParserContext, PState};

pub fn parse_with_recovery<'a, 'b: 'a, 'grm: 'b, O, E: ParseError<L = ErrorLabel<'grm>> + Clone>(
    sub: &'a impl Parser<'b, 'grm, O, E, PState<'b, 'grm, E>>,
    stream: StringStream<'grm>,
    cache: &mut PState<'b, 'grm, E>,
    context: &ParserContext<'b, 'grm>,
) -> Result<O, Vec<E>> {
    let mut recovery_points: HashMap<usize, usize> = HashMap::new();
    let mut result_errors = Vec::new();
    let mut last_err_pos: Option<usize> = None;
    let mut last_err_offset = 0usize;

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
                    Err(result_errors)
                }
            }
            PErr(e, s) => {
                //If this is the first time we encounter this, error, log it and retry
                if last_err_pos.is_none() || last_err_pos.unwrap() + last_err_offset < s.pos() {
                    result_errors.push(e);
                    last_err_pos = Some(s.pos());
                    last_err_offset = 0;
                    recovery_points.insert(s.pos(), last_err_offset);
                    continue;
                } else if let Some(last_err_pos) = last_err_pos {
                    //If the error now spans rest of file, we could not recover
                    let len_left = s.src().len() - s.pos();
                    if last_err_offset >= len_left {
                        return Err(result_errors)
                    }

                    //Increase offset by 1 and repeat
                    last_err_offset += 1;
                    recovery_points.insert(last_err_pos, last_err_offset);
                } else {
                    unreachable!()
                }
            }
        }
    }


}
