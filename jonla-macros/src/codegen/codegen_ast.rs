use crate::formatting_file::FormattingFile;
use crate::grammar::{Ast, AstType};
use itertools::Itertools;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::io::Write;

pub fn write_asts(mut file: FormattingFile, asts: &Vec<Ast>) {
    asts.iter().for_each(|ast| write_ast(&mut file, ast))
}

fn write_ast(file: &mut FormattingFile, ast: &Ast) {
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
                    let arg_type = process_type(arg_type, true);
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
        quote! {
            #[derive(Clone, Debug)]
            pub enum #name<'input> {
                #(#constrs),*
            }
        }
    )
    .unwrap();
}

pub(crate) fn process_type(typ: &AstType, need_box: bool) -> TokenStream {
    match typ {
        AstType::Input => {
            quote! { &'input str }
        }
        AstType::Ast(name) => {
            let name = format_ident!("{}", name);
            if need_box {
                quote! { Box<#name<'input>> }
            } else {
                quote! { #name<'input> }
            }
        }
        AstType::List(typ) => {
            let typ = process_type(typ, false);
            quote! { Vec<#typ> }
        }
    }
}
