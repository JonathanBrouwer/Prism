use crate::codegen::codegen_ast::process_type;
use crate::codegen::codegen_from_tuples::write_from_tuple_arg;
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
            use jonla_macros::parser::parser_rule::*;
            use jonla_macros::parser::action_result::*;
            use jonla_macros::parser::core::presult::*;
            use jonla_macros::parser::core::error::*;
            use jonla_macros::parser::core::stream::*;
            use jonla_macros::parser::core::primitives::*;
            use jonla_macros::parser::parser_state::*;
            use std::collections::HashMap;
            use jonla_macros::grammar::*;
            use jonla_macros::parser::core::parser::Parser;
            use jonla_macros::parser::error_printer::ErrorLabel;

            const RULES_STR: &'static str = include_str!("rules.json");
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
    let rtrn = process_type(&rule.rtrn, false);
    let from_action_result = write_from_tuple_arg(&rule.rtrn, quote!(&pr.1), false);

    write!(
        file,
        "{}",
        quote! {
            pub fn #name<'input>(input: &'input str) -> PResult<#rtrn, FullError<ErrorLabel<'_>>, StringStream<'input>> {
                let rules: HashMap<&'static str, RuleBody<'static>> = jonla_macros::read_rules_json(RULES_STR).unwrap();

                let mut state = ParserState::new();
                let stream: StringStream = input.into();
                let result: PResult<_, _, _> = full_input(&parser_rule::<StringStream<'_>, FullError<_>>(&rules, #name_str)).parse(stream, &mut state);

                result.map(|pr| #from_action_result)
            }
        }
    ).unwrap()
}
