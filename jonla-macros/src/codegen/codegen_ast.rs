use crate::formatting_file::FormattingFile;
use crate::grammar::Ast;
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::io::Write;

pub fn write_ast(mut file: FormattingFile, asts: &Vec<Ast>) {
    let asts: Vec<TokenStream> = asts
        .iter()
        .map(|ast| {
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
        })
        .collect_vec();
    write!(file, "{}", quote! { #(#asts)* }).unwrap();
}

fn process_type(name: &str) -> (TokenStream, bool) {
    if name == "Input" {
        (quote! { &'input str }, true)
    } else {
        let name = format_ident!("{}", name);
        (quote! { Box<#name<'input>> }, false)
    }
}
