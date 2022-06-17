use quote::{format_ident, quote};
use crate::formatting_file::FormattingFile;
use crate::grammar::Rule;
use std::io::Write;

pub fn write_parsers(mut file: FormattingFile, rules: &Vec<Rule>) {
    write!(
        file,
        "{}",
        quote! {
            use super::ast::*;
        }
    ).unwrap();

    rules.iter().for_each(|ast| write_parser(&mut file, ast))
}

fn write_parser(file: &mut FormattingFile, rule: &Rule) {
    let name = format_ident!("parse_{}", rule.name);
    let rtrn = format_ident!("{}", rule.rtrn);

    write!(
        file,
        "{}",
        quote! {
            pub fn #name() -> #rtrn {
                todo!()
            }
        }
    ).unwrap()
}