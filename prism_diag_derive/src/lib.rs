use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Data, DeriveInput, Expr, Fields, Ident, LitStr, Meta, Path, Token, Type, TypePath};
use synstructure::{Structure, decl_derive};

decl_derive!([Diagnostic, attributes(diag, sugg)] => derive_diagnostic);

fn derive_diagnostic(s: Structure<'_>) -> proc_macro::TokenStream {
    let Data::Struct(data) = &s.ast().data else {
        panic!("Expected struct")
    };
    let attr = s
        .ast()
        .attrs
        .iter()
        .find(|attr| attr.path().get_ident().unwrap().to_string().as_str() == "diag")
        .expect("Expected diag attr");
    let DiagArgs { title, id, env } = attr.parse_args().unwrap();

    let Fields::Named(named) = &data.fields else {
        panic!("Expected named fields")
    };

    let struct_name = &s.ast().ident;
    let diag_id = id.unwrap_or(struct_name_to_id(&struct_name.to_string()));

    let (env_param, env_generic) = match env {
        Some(env) => (None, quote!(#env)),
        None => (Some(quote!(__Env)), quote!(__Env)),
    };

    let mut groups: Vec<TokenStream> = vec![];
    for field in &named.named {
        if let Some(field_args) = field
            .attrs
            .iter()
            .find(|attr| attr.path().get_ident().unwrap().to_string().as_str() == "sugg")
        {
            let field_ident = field.ident.as_ref().unwrap();
            let sugg_args = if let Meta::List(..) = field_args.meta {
                field_args.parse_args::<SuggArgs>().unwrap()
            } else {
                SuggArgs::default()
            };

            let label = match sugg_args.label {
                Some(label) => quote!(Some(#label.to_string())),
                None => quote!(None),
            };

            groups.push(quote!{
                ::prism_diag::AnnotationGroup {
                    annotations: vec![
                        ::prism_diag::Annotation {
                            span: <_ as prism_diag::sugg::SuggestionArgument<#env_generic>>::span(&self.#field_ident, env),
                            label: #label
                        }
                    ]
                }
            })
        }
    }

    quote! {
        impl<#env_param> ::prism_diag::IntoDiag<#env_generic> for #struct_name {
            fn into_diag(self, env: &mut #env_generic) -> ::prism_diag::Diag {
                ::prism_diag::Diag {
                    title: #title,
                    id: #diag_id,
                    groups: vec![
                        #(#groups),*
                    ]
                }
            }
        }
    }
    .into()
}

fn struct_name_to_id(name: &str) -> String {
    let mut buffer = String::new();
    for char in name.chars() {
        if char.is_uppercase() && !buffer.is_empty() {
            buffer.push('_');
        }
        buffer.push_str(char.to_lowercase().to_string().as_str());
    }
    buffer
}

struct DiagArgs {
    title: Expr,
    id: Option<String>,
    env: Option<Type>,
}

impl Parse for DiagArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut title: Option<Expr> = None;
        let mut id: Option<String> = None;
        let mut env: Option<Type> = None;
        loop {
            let name = input.parse::<Ident>()?;
            input.parse::<Token![=]>()?;
            match name.to_string().as_str() {
                "title" => {
                    title = Some(input.parse::<Expr>()?);
                }
                "id" => {
                    id = Some(input.parse::<Ident>()?.to_string());
                }
                "env" => {
                    env = Some(input.parse::<Type>()?);
                }
                name => panic!("Unknown name: {name}"),
            }
            if input.is_empty() {
                break;
            }
            input.parse::<Token![,]>()?;
        }

        Ok(DiagArgs {
            title: title.expect("Expected `title` arg"),
            id,
            env,
        })
    }
}

#[derive(Default)]
struct SuggArgs {
    label: Option<Expr>,
}

impl Parse for SuggArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = Self::default();

        if !input.is_empty() {
            loop {
                let name = input.parse::<Ident>()?;
                input.parse::<Token![=]>()?;
                match name.to_string().as_str() {
                    "label" => {
                        args.label = Some(input.parse::<Expr>()?);
                    }
                    name => panic!("Unknown name: {name}"),
                }
                if input.is_empty() {
                    break;
                }
                input.parse::<Token![,]>()?;
            }
        }

        Ok(args)
    }
}
