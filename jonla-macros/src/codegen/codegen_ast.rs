use quote::{format_ident, quote};
use std::io::Write;
use itertools::Itertools;
use proc_macro2::{TokenStream};
use crate::formatting_file::FormattingFile;
use crate::grammar::Ast;

pub fn write_ast(mut file: FormattingFile, asts: &Vec<Ast>) {
    let asts: Vec<TokenStream> = asts.iter().map(|ast| {
        let name = format_ident!("{}", ast.name);
        let constrs = ast.constructors.iter().map(|cs| {
            let name = format_ident!("{}", cs.name);
            let args = cs.args.iter().map(|(arg_name, arg_type)| {
                let arg_name = format_ident!("{}", arg_name);
                let arg_type = process_type(arg_type);
                quote!(
                    #arg_name: #arg_type
                )
            }).collect_vec();
            quote! {
                #name { #(#args),* }
            }
        }).collect_vec();
        quote! {
            pub enum #name<'input> {
                #(#constrs),*
            }
        }
    }).collect_vec();
    write!(file, "{}", quote! { #(#asts)* }).unwrap();
}

fn process_type(name: &str) -> TokenStream {
    if name == "Input" {
        quote! { &'input str }
    } else {
        let name = format_ident!("{}", name);
        quote! { Box<#name<'input>> }
    }
}