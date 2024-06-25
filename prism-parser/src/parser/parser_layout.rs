use crate::core::adaptive::GrammarState;
use crate::core::context::ParserContext;
use crate::core::parser::Parser;
use crate::core::pos::Pos;
use crate::core::presult::PResult;
use crate::core::presult::PResult::{PErr, POk};
use crate::core::primitives::end;
use crate::core::state::PState;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::parser::parser_rule::parser_rule;
use crate::parser::var_map::VarMap;


pub fn parser_with_layout<
    'a,
    'arn: 'a,
    'grm: 'arn,
    O,
    E: ParseError<L = ErrorLabel<'grm>> + 'grm,
>(
    rules: &'arn GrammarState<'arn, 'grm>,
    vars: VarMap<'arn, 'grm>,
    sub: &'a impl Parser<'arn, 'grm, O, E>,
) -> impl Parser<'arn, 'grm, O, E> + 'a {
    move |pos: Pos, state: &mut PState<'arn, 'grm, E>, context: ParserContext| -> PResult<O, E> {
        if context.layout_disabled || vars.get("layout").is_none() {
            return sub.parse(pos, state, context);
        }
        let layout = vars
            .get("layout")
            .expect("Layout exists")
            .as_rule_id()
            .expect("Layout is an expr");

        //Start attemping to parse layout
        let mut res = PResult::new_empty((), pos);
        loop {
            let sub_res = sub.parse(res.end_pos(), state, context);
            if sub_res.is_ok() {
                return sub_res;
            }

            let pos_before_layout = sub_res.end_pos();
            // Add in optional error information from sub_res, then require another layout token
            let new_res = res.merge_seq_opt(sub_res).merge_seq_parser(
                &parser_rule(rules, layout, &[]),
                state,
                ParserContext {
                    layout_disabled: true,
                    ..context.clone()
                },
            );
            match new_res {
                // We have parsed more layout, we can try again
                POk(_, _, new_end_pos, new_err) if pos_before_layout < new_res.end_pos() => {
                    res = POk((), new_end_pos, new_end_pos, new_err);
                }
                // We have not parsed more layout ...
                // ... because layout parser did not parse more characters
                POk(_, _, _, err) => {
                    let (e, pos) = err.unwrap();
                    return PErr(e, pos);
                }
                // ... because layout parser failed
                PErr(e, pos) => return PErr(e, pos),
            }
        }
    }
}

pub fn full_input_layout<
    'a,
    'arn: 'a,
    'grm: 'arn,
    O,
    E: ParseError<L = ErrorLabel<'grm>> + 'grm,
>(
    rules: &'arn GrammarState<'arn, 'grm>,
    vars: VarMap<'arn, 'grm>,
    sub: &'a impl Parser<'arn, 'grm, O, E>,
) -> impl Parser<'arn, 'grm, O, E> + 'a {
    move |pos: Pos, state: &mut PState<'arn, 'grm, E>, context: ParserContext| -> PResult<O, E> {
        let res = sub.parse(pos, state, context);
        res.merge_seq_parser(&parser_with_layout(rules, vars, &end()), state, context)
            .map(|(o, _)| o)
    }
}
