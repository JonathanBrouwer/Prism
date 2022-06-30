use crate::formatting_file::FormattingFile;
use crate::grammar::{Ast, AstConstructor, AstType};
use itertools::Itertools;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use std::io::Write;

pub fn write_from_tuples(mut file: FormattingFile, asts: &Vec<Ast>) {
    write!(
        file,
        "{}",
        quote! {
            use super::ast::*;
            use jonla_macros::parser::parser_rule::*;
            use jonla_macros::parser::parser_core::*;
            use jonla_macros::parser::parser_result::*;
            use std::collections::HashMap;
            use jonla_macros::grammar::*;

            pub fn read_input<'grm: 'src, 'src>(ar: &ActionResult<'grm>, input: &'src str) -> &'src str {
                match ar {
                    ActionResult::Value((s, e)) => &input[*s..*e],
                    ActionResult::Literal(s) => s,
                    _ => unreachable!(),
                }
            }
        }
    )
    .unwrap();
    asts.iter().for_each(|ast| write_from_tuple(&mut file, ast))
}

fn write_from_tuple(file: &mut FormattingFile, ast: &Ast) {
    let funcname = format_ident!("{}_from_action_result", ast.name.to_lowercase());
    let returnname = format_ident!("{}", ast.name);

    let constructors = ast
        .constructors
        .iter()
        .map(|c| write_from_tuple_constructor(&returnname, c))
        .collect_vec();

    write!(
        file,
        "{}",
        quote! {
            pub fn #funcname<'grm: 'src, 'src>(a: &ActionResult<'grm>, input: &'src str) -> #returnname<'src> {
                match a {
                    ActionResult::Construct(name, args) => {
                        match *name {
                            #(#constructors),*,
                            _ => unreachable!(),
                        }
                    }
                    _ => unreachable!(),
                }
            }
        }
    ).unwrap()
}

fn write_from_tuple_constructor(sort: &Ident, cons: &AstConstructor) -> TokenStream {
    let cons_name_str = cons.name;
    let cons_name = format_ident!("{}", cons.name);

    let args: Vec<TokenStream> = cons
        .args
        .iter()
        .enumerate()
        .map(|(i, (an, av))| {
            let an = format_ident!("{}", an);
            let av = write_from_tuple_arg(i, av);
            quote! {
                #an: #av
            }
        })
        .collect();

    quote! {
        #cons_name_str => #sort::#cons_name{ #(#args),* }
    }
}

fn write_from_tuple_arg(i: usize, arg: &AstType) -> TokenStream {
    match arg {
        AstType::Input => {
            quote! {
            read_input(&args[#i], input)
        }
        }
        AstType::Rule(rule) => {
            let funcname = format_ident!("{}_from_action_result", rule.to_lowercase());
            quote! {
                Box::new(#funcname(&args[#i], input))
            }
        }
        AstType::List(vals) => {
            quote! {
                match &args[#i] {
                    ActionResult::List(args) => {
                        todo!()
                    },
                    _ => unreachable!(),
                }
            }
        }
    }
}
