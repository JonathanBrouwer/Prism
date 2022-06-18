use crate::formatting_file::FormattingFile;
use crate::grammar::Rule;
use quote::{format_ident, quote};
use std::io::Write;

pub fn write_parsers(mut file: FormattingFile, rules: &Vec<Rule>) {
    write!(
        file,
        "{}",
        quote! {
            use super::ast::*;
            use super::from_tuples::*;
            use jonla_macros::parser::parser_core::*;
            use jonla_macros::parser::parser_result::*;
            use jonla_macros::parser::parser_rule::*;
            use std::collections::HashMap;
            use jonla_macros::grammar::*;
        }
    )
    .unwrap();

    rules.iter().for_each(|ast| write_parser(&mut file, ast))
}

fn write_parser(file: &mut FormattingFile, rule: &Rule) {
    if rule.name.starts_with("_") {
        return;
    }

    let name_str = rule.name;
    let name = format_ident!("parse_{}", rule.name);
    let rtrn = format_ident!("{}", rule.rtrn);
    let rtrn = if rule.rtrn == "Input" {
        quote! { &'input str }
    } else {
        quote! { #rtrn<'input> }
    };

    write!(
        file,
        "{}",
        quote! {
            pub fn #name<'input>(inp: &'input str) -> ParseResult<#rtrn> {
                let str: &'static str = include_str!("rules.json");
                let rules: HashMap<&'static str, RuleBody<'static>> = jonla_macros::read_rules_json(str).unwrap();
                let mut state: ParserState = ParserState::new(inp);
                let result: ParseResult<PR> = state.parse_rule(0, &rules, #name_str);
                result.map(|pr| test_from_action_result(&pr.1, inp))
            }
        }
    ).unwrap()
}
