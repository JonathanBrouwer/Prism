use crate::core::adaptive::GrammarState;
use crate::core::cache::PCache;
use crate::core::context::ParserContext;
use crate::core::parser::Parser;
use crate::core::pos::Pos;
use crate::core::presult::PResult;
use crate::core::presult::PResult::{PErr, POk};
use crate::core::primitives::end;
use crate::error::error_printer::ErrorLabel;
use crate::error::ParseError;
use crate::grammar::parser_rule::parser_rule;

pub fn parser_with_layout<'a, 'b: 'a, 'grm: 'b, O, E: ParseError<L = ErrorLabel<'grm>> + Clone>(
    rules: &'b GrammarState<'b, 'grm>,
    sub: &'a impl Parser<'b, 'grm, O, E>,
) -> impl Parser<'b, 'grm, O, E> + 'a {
    move |pos: Pos,
          cache: &mut PCache<'b, 'grm, E>,
          context: &ParserContext<'b, 'grm>|
          -> PResult<O, E> {
        if context.layout_disabled || !rules.contains_rule("layout") {
            return sub.parse(pos, cache, context);
        }

        //Start attemping to parse layout
        let mut res = PResult::new_ok((), pos);
        loop {
            let (new_res, success) = res.merge_seq_opt_parser(sub, cache, context);
            if success {
                return new_res.map(|(_, o)| o.unwrap());
            }

            let pos_before_layout = new_res.get_pos();
            let new_res = new_res.merge_seq_parser(
                &parser_rule(rules, "layout"),
                cache,
                &ParserContext {
                    layout_disabled: true,
                    ..context.clone()
                },
            );
            match new_res {
                POk(_, _, _) if pos_before_layout < new_res.get_pos() => {
                    res = new_res.map(|_| ());
                }
                POk(_, _, _) => {
                    return new_res
                        .merge_seq_parser(sub, cache, context)
                        .map(|(_, r)| r);
                }
                PErr(_, _) => {
                    return new_res.map(|_| unreachable!());
                }
            }
        }
    }
}

pub fn full_input_layout<'a, 'b: 'a, 'grm: 'b, O, E: ParseError<L = ErrorLabel<'grm>> + Clone>(
    rules: &'b GrammarState<'b, 'grm>,
    sub: &'a impl Parser<'b, 'grm, O, E>,
) -> impl Parser<'b, 'grm, O, E> + 'a {
    move |stream: Pos,
          cache: &mut PCache<'b, 'grm, E>,
          context: &ParserContext<'b, 'grm>|
          -> PResult<O, E> {
        let res = sub.parse(stream, cache, context);
        res.merge_seq_parser(&parser_with_layout(rules, &end()), cache, context)
            .map(|(o, _)| o)
    }
}
