mod lexer;

use logos::{Span, Source};
use crate::lexer::{LexerToken, JonlaLexer};

fn main() {
    let source = include_str!("../resources/test_indent.jnl");

    let lexer = JonlaLexer::lexer(source);
    let lexer_tokens : Vec<(LexerToken, Span)> = lexer.collect();

    let print_lexer_result = true;
    if print_lexer_result {
        println!("------------------");
        println!("Lexer tokens:");

        lexer_tokens.iter().for_each(|(t, s)| {
            println!("{:?} - {}", t, source.slice(s.clone()).unwrap().escape_debug());
        });
    }




}
