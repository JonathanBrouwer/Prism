use crate::codegen::codegen_ast::process_type;
use crate::formatting_file::FormattingFile;
use crate::grammar::{Rule};
use quote::{format_ident, quote};
use std::io::Write;
use crate::codegen::codegen_from_tuples::write_from_tuple_arg;

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
            pub fn #name<'input>(input: &'input str) -> ParseResult<'static, #rtrn> {
                let rules: HashMap<&'static str, RuleBody<'static>> = jonla_macros::read_rules_json(RULES_STR).unwrap();
                let mut state: ParserState<'static, 'input, PR<'static>> = ParserState::new(input);
                let result: ParseResult<'static, PR<'static>> = state.parse_full_input(|s, p| s.parse_rule(p, &rules, #name_str));
                result.map(|pr| #from_action_result)
            }
        }
    ).unwrap()
}
