use crate::formatting_file::FormattingFile;
use crate::grammar::Ast;
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::io::Write;

pub fn write_asts(mut file: FormattingFile, asts: &Vec<Ast>) {
    asts.iter().for_each(|ast| write_ast(&mut file, ast))
}

fn write_ast(file: &mut FormattingFile, ast: &Ast) {
    let mut used_input = false;
    let name = format_ident!("{}", ast.name);
    let constrs = ast
        .constructors
        .iter()
        .map(|cs| {
            let name = format_ident!("{}", cs.name);
            let args = cs
                .args
                .iter()
                .map(|(arg_name, arg_type)| {
                    let arg_name = format_ident!("{}", arg_name);
                    let (arg_type, l) = process_type(arg_type);
                    used_input |= l;
                    quote!(
                        #arg_name: #arg_type
                    )
                })
                .collect_vec();
            quote! {
                #name { #(#args),* }
            }
        })
        .collect_vec();
    write!(
        file,
        "{}",
        if used_input {
            quote! {
                pub enum #name<'input> {
                    #(#constrs),*
                }
            }
        } else {
            quote! {
                pub enum #name {
                    #(#constrs),*
                }
            }
        }
    )
    .unwrap();
}

fn process_type(name: &str) -> (TokenStream, bool) {
    if name == "Input" {
        (quote! { &'input str }, true)
    } else {
        let name = format_ident!("{}", name);
        (quote! { Box<#name<'input>> }, false)
    }
}
