use std::rc::Rc;
use itertools::Itertools;
use crate::lambday::lambday::LambdayTerm;
use crate::{Parser, ParseSuccess};
use crate::peg::input::Input;
use crate::peg::parsers::alt::alt;
use crate::peg::parsers::complete_input::complete_input;
use crate::peg::parsers::exact::*;
use crate::peg::parsers::ignore_whitespace::ignore_whitespace;
use crate::peg::parsers::repeat_m_n::{repeat_m_n, repeat_m_n_matching};
use crate::peg::parsers::seq::*;

pub fn parse_jonla_program<I: Input<InputElement=char>>() -> impl Parser<I, Vec<LambdayTerm<String>>> {
    complete_input(repeat_m_n(parse_jonla_term(), 0, usize::MAX))
}

pub fn parse_jonla_term<I: Input<InputElement=char>>() -> impl Parser<I, LambdayTerm<String>> {
    move |pos: I| {
        let ok = parse_lamday_term().parse(pos)?;
        Ok(ok)
    }
}

pub fn parse_lamday_term<I: Input<InputElement=char>>() -> impl Parser<I, LambdayTerm<String>> {
    move |pos: I| {
        let parsers: Vec<Box<dyn Parser<I, LambdayTerm<String>>>> = vec![
            //Prod type
            Box::new(|pos: I| {
                let ok = seq4ws(
                    exact_str("#pt"),
                    exact_str("("),
                    repeat_m_n(parse_jonla_term(), 0, usize::MAX),
                    exact_str(")")
                ).parse(pos)?;
                let (_, _, types, _) = ok.result;

                Ok(ParseSuccess {
                    result: LambdayTerm::ProdType(types.into_iter().map(|t| Rc::new(t)).collect()),
                    best_error: ok.best_error,
                    pos: ok.pos
                })
            }),
            //Prod construct
            Box::new(|pos: I| {
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
                    result: LambdayTerm::ProdConstr(Rc::new(typ), vals.into_iter().map(|t| Rc::new(t)).collect()),
                    best_error: ok.best_error,
                    pos: ok.pos
                })
            }),
            //Prod destruct
            Box::new(|pos: I| {
                let ok = seq5ws(
                    exact_str("#pd"),
                    exact_str("("),
                    parse_jonla_term(),
                    exact_str(")"),
                    parse_usize()
                ).parse(pos)?;
                let (_, _, val, _, num) = ok.result;

                Ok(ParseSuccess {
                    result: LambdayTerm::ProdDestr(Rc::new(val), num),
                    best_error: ok.best_error,
                    pos: ok.pos
                })
            }),
            //Sum type
            Box::new(|pos: I| {
                let ok = seq4ws(
                    exact_str("#st"),
                    exact_str("("),
                    repeat_m_n(parse_jonla_term(), 0, usize::MAX),
                    exact_str(")")
                ).parse(pos)?;
                let (_, _, types, _) = ok.result;

                Ok(ParseSuccess {
                    result: LambdayTerm::SumType(types.into_iter().map(|t| Rc::new(t)).collect()),
                    best_error: ok.best_error,
                    pos: ok.pos
                })
            }),
            //Sum construct
            Box::new(|pos: I| {
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
                    result: LambdayTerm::SumConstr(Rc::new(typ), num, Rc::new(val)),
                    best_error: ok.best_error,
                    pos: ok.pos
                })
            }),
            //Sum destruct
            Box::new(|pos: I| {
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
                        Rc::new(val),
                        Rc::new(rt),
                        opts.into_iter().map(|t| Rc::new(t)).collect()
                    ),
                    best_error: ok.best_error,
                    pos: ok.pos
                })
            }),
            //Fun type
            Box::new(|pos: I| {
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
                    result: LambdayTerm::FunType(Rc::new(t1), Rc::new(t2)),
                    best_error: ok.best_error,
                    pos: ok.pos
                })
            }),
            //Fun construct
            Box::new(|pos: I| {
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
                    result: LambdayTerm::FunConstr(t1, Rc::new(t2), Rc::new(t3)),
                    best_error: ok.best_error,
                    pos: ok.pos
                })
            }),
            //Fun destruct
            Box::new(|pos: I| {
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
                    result: LambdayTerm::FunDestr(Rc::new(t1), Rc::new(t2)),
                    best_error: ok.best_error,
                    pos: ok.pos
                })
            }),
            //Type type
            Box::new(|input: I| {
                let ok = ignore_whitespace(exact_str("#tt")).parse(input)?;
                Ok(ParseSuccess {
                    result: LambdayTerm::TypeType(),
                    best_error: ok.best_error,
                    pos: ok.pos
                })
            }),
            //Var
            Box::new(|input: I| {
                ignore_whitespace(parse_name()).parse(input).map(|s| s.map(|r| LambdayTerm::Var(r)))
            }),
        ];
        alt(parsers).parse(pos.clone())
    }
}

pub fn parse_name<I: Input<InputElement=char>>() -> impl Parser<I, String> {
    move |pos: I| {
        repeat_m_n_matching(
            "name".to_string(),
            Box::new(|c: char| c.is_alphabetic()),
            1, usize::MAX).parse(pos)
            .map(|s| s.map(|r| r.into_iter().collect()))
    }
}

pub fn parse_usize<I: Input<InputElement=char>>() -> impl Parser<I, usize> {
    move |pos: I| {
        repeat_m_n_matching(
            "number".to_string(),
            Box::new(|c: char| c.is_ascii_digit()),
            1, usize::MAX).parse(pos)
            .map(|s| s.map(|r| r.into_iter().collect::<String>().parse().unwrap()))
    }
}