use crate::core::state::PState;
use crate::core::context::{Ignore, ParserContext};
use crate::core::cow::Cow;
use crate::core::parser::Parser;
use crate::core::pos::Pos;
use crate::core::presult::PResult;
use crate::core::presult::PResult::{PErr, POk};
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::rule_action::action_result::ActionResult;
use std::collections::HashMap;
use std::sync::Arc;

const MAX_RECOVERIES: usize = 5;

pub fn parse_with_recovery<'a, 'arn: 'a, 'grm: 'arn, O, E: ParseError<L = ErrorLabel<'grm>>>(
    sub: &'a impl Parser<'arn, 'grm, O, E>,
    pos: Pos,
    state: &mut PState<'arn, 'grm, E>,
    context: &ParserContext,
) -> Result<O, Vec<E>> {
    let mut recovery_points: HashMap<Pos, Pos> = HashMap::new();
    let mut result_errors: Vec<E> = Vec::new();
    let mut err_state: Option<(Pos, Pos)> = None;

    loop {
        let context = ParserContext {
            recovery_points: Ignore(Arc::new(recovery_points.clone())),
            ..context.clone()
        };

        match sub.parse(pos, state, &context) {
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

                    // Check if we can accept more errors
                    if result_errors.len() >= MAX_RECOVERIES {
                        return Err(result_errors);
                    }

                    // Add new error
                    result_errors.push(e);
                    err_state = Some((p, p));
                } else if let Some((_err_state_start, err_state_end)) = &mut err_state {
                    //If the error now spans rest of file, we could not recover
                    if *err_state_end == Pos::end(state.input) {
                        result_errors
                            .last_mut()
                            .unwrap()
                            .set_end(Pos::end(state.input));
                        return Err(result_errors);
                    }

                    //Increase offset by one char and repeat
                    *err_state_end = err_state_end.next(state.input).0;
                    debug_assert!(*err_state_end <= Pos::end(state.input));
                } else {
                    unreachable!()
                }
                recovery_points.insert(err_state.unwrap().0, err_state.unwrap().1);
                recovery_points.insert(err_state.unwrap().1, err_state.unwrap().1);
                state.clear();
            }
        }
    }
}

pub fn recovery_point<'a, 'arn: 'a, 'grm: 'arn, E: ParseError<L = ErrorLabel<'grm>> + 'arn>(
    item: impl Parser<'arn, 'grm, Cow<'arn, ActionResult<'arn, 'grm>>, E> + 'a,
) -> impl Parser<'arn, 'grm, Cow<'arn, ActionResult<'arn, 'grm>>, E> + 'a {
    move |pos: Pos,
          state: &mut PState<'arn, 'grm, E>,
          context: &ParserContext|
          -> PResult<_, E> {
        // First try original parse
        match item.parse(
            pos,
            state,
            &ParserContext {
                recovery_disabled: true,
                ..context.clone()
            },
        ) {
            r @ POk(_, _, _, _, _) => r,
            PErr(e, s) => {
                if let Some(to) = context.recovery_points.get(&s) {
                    POk(
                        Cow::Owned(ActionResult::void()),
                        pos,
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
