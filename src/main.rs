use logos::Source;

use crate::parser::parser::JonlaParser;
use crate::lexer::lexer::{JonlaLexer, LexerToken};

mod parser;
mod lexer;

fn main() {
    let source = include_str!("../resources/test_fn.jnl");

    let lexer = JonlaLexer::lexer(source);
    let lexer_tokens : Vec<LexerToken> = lexer.collect();

    let print_lexer_result = false;
    if print_lexer_result {
        println!("------------------");
        println!("Lexer tokens:");

        lexer_tokens.iter().for_each(|t| {
            println!("{:?} - {}", t, source.slice(t.span.clone()).unwrap().escape_debug());
        });
    }

    let mut parser = JonlaParser::new(source, lexer_tokens);
    let program = parser.parse_program();
    let program = match program {
        Ok(program) => program,
        Err(err) => {
            println!("Parse error: {}", err);
            return
        }
    };

    let print_parser_result = true;
    if print_parser_result {
        println!("------------------");
        println!("Parser output:");

        println!("{}", program)
    }

}
