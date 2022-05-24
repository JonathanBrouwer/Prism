use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use proc_macro2::TokenStream;
use quote::quote;
use crate::codegen::codegen_ast::write_ast;
use crate::formatting_file::FormattingFile;
use crate::GrammarFile;

mod codegen_ast;

pub fn codegen(grammar: &GrammarFile) {
    let [mod_file, ast_file] = verify_folder_structure();
    write_mod(mod_file);
    write_ast(ast_file, &grammar.asts);
}

fn verify_folder_structure() -> [FormattingFile; 2] {
    let folder: PathBuf = "src/autogen".into();
    let _ = std::fs::remove_dir_all(&folder);
    std::fs::create_dir(&folder).unwrap();

    {
        let mut file = folder.clone();
        file.push(".gitignore");
        let file = File::create(file).unwrap();
        write_gitignore(file);
    }

    ["mod.rs", "ast.rs"].map(| filename | {
        let mut file = folder.clone();
        file.push(filename);
        FormattingFile::create(file).unwrap()
    })
}

fn write_mod(mut file: FormattingFile) {
    let tokens: TokenStream = quote! {
        pub mod ast;
    };
    write!(file, "{}", tokens).unwrap();
}

fn write_gitignore(mut file: File) {
    write!(file, "/**").unwrap();
}