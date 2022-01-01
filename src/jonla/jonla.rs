use crate::lambday::lambday::LambdayTerm;
use crate::{Parser, ParseSuccess};
use crate::jonla::jerror::Span;
use crate::peg::input::{Input};
use crate::peg::parsers::alt::alt;
use crate::peg::parsers::complete_input::complete_input;
use crate::peg::parsers::exact::*;
use crate::peg::parsers::repeat_m_n::{repeat_m_n, repeat_m_n_matching};
use crate::peg::parsers::seq::*;

pub fn parse_jonla_program<'a>() -> impl Parser<'a, LambdayTerm<Span, String>> {
    complete_input(parse_jonla_term())
}

pub fn parse_jonla_term<'a>() -> impl Parser<'a, LambdayTerm<Span, String>> {
    move |pos: Input| {
        let ok = parse_lamday_term().parse(pos)?;
        Ok(ok)
    }
}

pub fn parse_lamday_term<'a>() -> impl Parser<'a, LambdayTerm<Span, String>> {
    move |startpos: Input<'a>| {
        let parsers: Vec<Box<dyn Fn(_) -> _>> = vec![
            //Prod type
            Box::new(|pos: Input<'a>| {
                let ok = seq4ws(
                    exact_str("#pt"),
                    exact_str("("),
                    repeat_m_n(parse_jonla_term(), 0, usize::MAX),
                    exact_str(")")
                ).parse(pos)?;
                let (_, _, types, _) = ok.result;

                Ok(ParseSuccess {
                    result: LambdayTerm::ProdType((pos.pos, ok.pos), types),
                    best_error: ok.best_error,
                    pos: ok.pos
                })
            }),
            //Prod construct
            Box::new(|pos: Input<'a>| {
                let ok = seq7ws(
                    exact_str("#pc"),
                    exact_str("("),
                    parse_jonla_term(),
                    exact_str(")"),
                    exact_str("("),
                    repeat_m_n(parse_jonla_term(), 0, usize::MAX),
                    exact_str(")")
                ).parse(pos)?;
                let (_, _, typ, _, _, vals, _) = ok.result;

                Ok(ParseSuccess {
                    result: LambdayTerm::ProdConstr((pos.pos, ok.pos), Box::new(typ), vals),
                    best_error: ok.best_error,
                    pos: ok.pos
                })
            }),
            //Prod destruct
            Box::new(|pos: Input| {
                let ok = seq5ws(
                    exact_str("#pd"),
                    exact_str("("),
                    parse_jonla_term(),
                    exact_str(")"),
                    parse_usize()
                ).parse(pos)?;
                let (_, _, val, _, num) = ok.result;

                Ok(ParseSuccess {
                    result: LambdayTerm::ProdDestr((pos.pos, ok.pos), Box::new(val), num),
                    best_error: ok.best_error,
                    pos: ok.pos
                })
            }),
            //Sum type
            Box::new(|pos: Input| {
                let ok = seq4ws(
                    exact_str("#st"),
                    exact_str("("),
                    repeat_m_n(parse_jonla_term(), 0, usize::MAX),
                    exact_str(")")
                ).parse(pos)?;
                let (_, _, types, _) = ok.result;

                Ok(ParseSuccess {
                    result: LambdayTerm::SumType((pos.pos, ok.pos), types),
                    best_error: ok.best_error,
                    pos: ok.pos
                })
            }),
            //Sum construct
            Box::new(|pos: Input| {
                let ok = seq8ws(
                    exact_str("#sc"),
                    exact_str("("),
                    parse_jonla_term(),
                    exact_str(")"),
                    parse_usize(),
                    exact_str("("),
                    parse_jonla_term(),
                    exact_str(")"),
                ).parse(pos)?;
                let (_, _, typ, _, num, _, val, _) = ok.result;

                Ok(ParseSuccess {
                    result: LambdayTerm::SumConstr((pos.pos, ok.pos), Box::new(typ), num, Box::new(val)),
                    best_error: ok.best_error,
                    pos: ok.pos
                })
            }),
            //Sum destruct
            Box::new(|pos: Input| {
                let ok = seq10ws(
                    exact_str("#sd"),
                    exact_str("("),
                    parse_jonla_term(),
                    exact_str(")"),
                    exact_str("("),
                    parse_jonla_term(),
                    exact_str(")"),
                    exact_str("("),
                    repeat_m_n(parse_jonla_term(), 0, usize::MAX),
                    exact_str(")"),
                ).parse(pos)?;
                let (_, _, val, _, _, rt, _, _, opts, _) = ok.result;

                Ok(ParseSuccess {
                    result: LambdayTerm::SumDestr(
                        (pos.pos, ok.pos),
                        Box::new(val),
                        Box::new(rt),
                        opts
                    ),
                    best_error: ok.best_error,
                    pos: ok.pos
                })
            }),
            //Fun type
            Box::new(|pos: Input| {
                let ok = seq6ws(
                    exact_str("#ft"),
                    exact_str("("),
                    parse_jonla_term(),
                    exact_str("->"),
                    parse_jonla_term(),
                    exact_str(")")
                ).parse(pos)?;
                let (_, _, t1, _, t2, _) = ok.result;

                Ok(ParseSuccess {
                    result: LambdayTerm::FunType((pos.pos, ok.pos), Box::new(t1), Box::new(t2)),
                    best_error: ok.best_error,
                    pos: ok.pos
                })
            }),
            //Fun construct
            Box::new(|pos: Input| {
                let ok = seq9ws(
                    exact_str("#fc"),
                    exact_str("("),
                    parse_name(),
                    exact_str(":"),
                    parse_jonla_term(),
                    exact_str(")"),
                    exact_str("("),
                    parse_jonla_term(),
                    exact_str(")")
                ).parse(pos)?;
                let (_, _, t1, _, t2, _, _, t3, _) = ok.result;

                Ok(ParseSuccess {
                    result: LambdayTerm::FunConstr((pos.pos, ok.pos), t1, Box::new(t2), Box::new(t3)),
                    best_error: ok.best_error,
                    pos: ok.pos
                })
            }),
            //Fun destruct
            Box::new(|pos: Input| {
                let ok = seq7ws(
                    exact_str("#fd"),
                    exact_str("("),
                    parse_jonla_term(),
                    exact_str(")"),
                    exact_str("("),
                    parse_jonla_term(),
                    exact_str(")")
                ).parse(pos)?;
                let (_, _, t1, _, _, t2, _) = ok.result;

                Ok(ParseSuccess {
                    result: LambdayTerm::FunDestr((pos.pos, ok.pos), Box::new(t1), Box::new(t2)),
                    best_error: ok.best_error,
                    pos: ok.pos
                })
            }),
            //Type type
            Box::new(|pos: Input| {
                let ok = seq1ws(
                    exact_str("#tt")
                ).parse(pos)?;
                let _ = ok.result;

                Ok(ParseSuccess {
                    result: LambdayTerm::TypeType((pos.pos, ok.pos)),
                    best_error: ok.best_error,
                    pos: ok.pos
                })
            }),
            //Var
            Box::new(|pos: Input| {
                let ok = seq1ws(
                    parse_name()
                ).parse(pos)?;
                let name = ok.result;

                Ok(ParseSuccess {
                    result: LambdayTerm::Var((pos.pos, ok.pos), name),
                    best_error: ok.best_error,
                    pos: ok.pos
                })
            }),
        ];
        alt(parsers).parse(startpos.clone())
    }
}

pub fn parse_name<'a>() -> impl Parser<'a, String> {
    move |pos: Input| {
        repeat_m_n_matching(
            "name".to_string(),
            Box::new(|c: char| c.is_alphabetic()),
            1, usize::MAX).parse(pos)
            .map(|s| s.map(|r| r.into_iter().collect()))
    }
}

pub fn parse_usize<'a>() -> impl Parser<'a, usize> {
    move |pos: Input| {
        repeat_m_n_matching(
            "number".to_string(),
            Box::new(|c: char| c.is_ascii_digit()),
            1, usize::MAX).parse(pos)
            .map(|s| s.map(|r| r.into_iter().collect::<String>().parse().unwrap()))
    }
}