use std::collections::HashMap;
use std::sync::Arc;
use crate::core::adaptive::GrammarState;
use crate::core::cache::PCache;
use crate::core::context::{ParserContext, RawEnv};
use crate::core::parser::Parser;
use crate::core::pos::Pos;
use crate::core::presult::PResult;
use crate::core::presult::PResult::{PErr, POk};
use crate::core::primitives::end;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::grammar::grammar::Action;
use crate::grammar::parser_rule::parser_rule;

pub fn parser_with_layout<'a, 'b: 'a, 'grm: 'b, O, E: ParseError<L = ErrorLabel<'grm>> + Clone, A: Action<'grm>>(
    rules: &'b GrammarState<'b, 'grm>,
    vars: &'a HashMap<&'grm str, Arc<RawEnv<'b, 'grm, A>>>,
    sub: &'a impl Parser<'b, 'grm, O, E>,
) -> impl Parser<'b, 'grm, O, E> + 'a {
    move |pos: Pos,
          cache: &mut PCache<'b, 'grm, E>,
          context: &ParserContext|
          -> PResult<O, E> {
        if context.layout_disabled || !vars.contains_key("layout") {
            return sub.parse(pos, cache, context);
        }
        let layout = A::eval_to_rule(&vars["layout"]).expect("Expected layout to be a rule.");

        //Start attemping to parse layout
        let mut res = PResult::new_empty((), pos);
        loop {
            let sub_res = sub.parse(res.end_pos(), cache, context);
            if sub_res.is_ok() {
                return sub_res;
            }

            let pos_before_layout = sub_res.end_pos();
            // Add in optional error information from sub_res, then require another layout token
            let new_res = res.merge_seq_opt(sub_res).merge_seq_parser(
                &parser_rule(rules, layout, &vec![]),
                cache,
                &ParserContext {
                    layout_disabled: true,
                    ..context.clone()
                },
            );
            match new_res {
                // We have parsed more layout, we can try again
                POk(_, _, new_end_pos, empty, new_err) if pos_before_layout < new_res.end_pos() => {
                    res = POk((), new_end_pos, new_end_pos, empty, new_err);
                }
                // We have not parsed more layout ...
                // ... because layout parser did not parse more characters
                POk(_, _, _, _, err) => {
                    let (e, pos) = err.unwrap();
                    return PErr(e, pos);
                }
                // ... because layout parser failed
                PErr(e, pos) => return PErr(e, pos),
            }
        }
    }
}

pub fn full_input_layout<'a, 'b: 'a, 'grm: 'b, O, E: ParseError<L = ErrorLabel<'grm>> + Clone, A: Action<'grm>>(
    rules: &'b GrammarState<'b, 'grm>,
    vars: &'a HashMap<&'grm str, Arc<RawEnv<'b, 'grm, A>>>,
    sub: &'a impl Parser<'b, 'grm, O, E>,
) -> impl Parser<'b, 'grm, O, E> + 'a {
    move |stream: Pos,
          cache: &mut PCache<'b, 'grm, E>,
          context: &ParserContext|
          -> PResult<O, E> {
        let res = sub.parse(stream, cache, context);
        res.merge_seq_parser(&parser_with_layout(rules, vars, &end()), cache, context)
            .map(|(o, _)| o)
    }
}
