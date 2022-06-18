use std::collections::HashMap;
use quote::{format_ident, quote};
use crate::formatting_file::FormattingFile;
use crate::grammar::{Rule, RuleBody};
use std::io::Write;
use lazy_static::lazy_static;
use proc_macro2::TokenStream;

// pub fn parse_term<'input>(s: &'input str) -> ParseResult<Term<'input>> {
//     let mut state = ParserState::new()
// }

pub fn write_parsers<'input>(mut file: FormattingFile, rules: &Vec<Rule<'input>>) {
    write!(
        file,
        "{}",
        quote! {
            use super::ast::*;
            use jonla_macros::parser::parser_core::*;
            use jonla_macros::parser::parser_result::*;
            use std::collections::HashMap;
            use jonla_macros::grammar::*;
        }
    ).unwrap();

    rules.iter().for_each(|ast| write_parser(&mut file, ast))
}

fn write_parser(file: &mut FormattingFile, rule: &Rule) {
    if rule.name.starts_with("_") {
        return;
    }

    let name_str = rule.name;
    let name = format_ident!("parse_{}", rule.name);
    let rtrn = format_ident!("{}", rule.rtrn);
    let rtrn = if rule.rtrn == "Input" {quote!{ &'input str }} else { quote!{ #rtrn<'input> } };

    write!(
        file,
        "{}",
        quote! {
            pub fn #name<'input>(inp: &'input str) -> ParseResult<#rtrn> {
                let str: &'static str = include_str!("rules.json");
                let rules: HashMap<&'static str, RuleBody<'static>> = jonla_macros::read_rules_json(str).unwrap();
                let mut state = ParserState::new(inp);
                let result = state.parse_rule(0, &rules, #name_str);

                todo!()
            }
        }
    ).unwrap()
}

fn process_return_type(name: &str) -> (TokenStream, bool) {
    if name == "Input" {
        (quote! { &'input str }, true)
    } else {
        let name = format_ident!("{}", name);
        (quote! { #name<'input> }, false)
    }
}