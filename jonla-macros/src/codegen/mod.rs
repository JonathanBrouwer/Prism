use crate::codegen::codegen_ast::write_asts;
use crate::codegen::codegen_parse::write_parsers;
use crate::formatting_file::FormattingFile;
use crate::GrammarFile;
use proc_macro2::TokenStream;
use quote::quote;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use crate::codegen::codegen_from_tuples::write_from_tuples;
use crate::codegen::codegen_rules::write_rules;

mod codegen_ast;
mod codegen_parse;
mod codegen_rules;
mod codegen_from_tuples;

pub fn codegen(grammar: &GrammarFile) {
    let [mod_file, ast_file, from_tuples_file, rules_file, parse_file] = verify_folder_structure();
    write_mod(mod_file);
    write_asts(ast_file, &grammar.asts);
    write_from_tuples(from_tuples_file, &grammar.asts);
    write_rules(rules_file, &grammar.rules);
    write_parsers(parse_file, &grammar.rules);
}

fn verify_folder_structure() -> [FormattingFile; 5] {
    let folder: PathBuf = "src/autogen".into();
    let _ = std::fs::remove_dir_all(&folder);
    std::fs::create_dir(&folder).unwrap();

    {
        let mut file = folder.clone();
        file.push(".gitignore");
        let file = File::create(file).unwrap();
        write_gitignore(file);
    }

    ["mod.rs", "ast.rs", "from_tuples.rs", "rules.json", "parse.rs"].map(|filename| {
        let mut file = folder.clone();
        file.push(filename);
        if filename.ends_with(".rs") {
            FormattingFile::create_formatting(file).unwrap()
        }else {
            FormattingFile::create_not_formatting(file).unwrap()
        }
    })
}

fn write_mod(mut file: FormattingFile) {
    let tokens: TokenStream = quote! {
        #[allow(unused)]
        pub mod ast;
        #[allow(unused)]
        pub mod from_tuples;
        #[allow(unused)]
        pub mod parse;
    };
    write!(file, "{}", tokens).unwrap();
}

fn write_gitignore(mut file: File) {
    write!(file, "/**").unwrap();
}
