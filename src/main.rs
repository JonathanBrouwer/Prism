use crate::lexer::lexer::*;
use logos::{Source};
use crate::lexer::layout_builder::LayoutBuilder;

mod lexer;

fn main() {
    let source = include_str!("../resources/test_block.jnl");

    // Lexer
    let lexer = FinalLexer::new(source);
    let (lexer_res, lexer_err) = lexer.collect_and_errors();

    let print_lexer_result = true;
    if print_lexer_result {
        println!("------------------");
        println!("Lexer tokens:");

        lexer_res.iter().for_each(|l| {
            print!("{} - ", l.indent);
            for token in &l.tokens {
                print!("{} ", source.slice(token.span.clone()).unwrap().escape_debug());
            }
            println!();
        });
    }

    if lexer_err.len() > 0 {
        println!("------------------");
        println!("Lexer errors:");
        lexer_err.iter().for_each(|e| {
            println!("{:?} - {:?}", e, &source[e.start..e.end]);
        });
        return
    }

    // Layout

    let layout = LayoutBuilder { input: lexer_res };
    let (layout_res, layout_err) = layout.build_layout();

    let print_layout_result = true;
    if print_layout_result {
        println!("------------------");
        println!("Layout tokens:");
        for item in &layout_res {
            println!("{:?}", item);
        }

    }

    if layout_err.len() > 0 {
        println!("------------------");
        println!("Layout errors:");
        layout_err.iter().for_each(|e| {
            println!("{:?} - {:?}", e, &source[e.start..e.end]);
        });
        return
    }




    // let mut parser = JonlaParser::new(source, lexer_tokens);
    // let program = parser.parse_program();
    // let program = match program {
    //     Ok(program) => program,
    //     Err(err) => {
    //         println!("Parse errors:\n{}", err);
    //         return;
    //     }
    // };
    //
    // let print_parser_result = true;
    // if print_parser_result {
    //     println!("------------------");
    //     println!("Parser output:");
    //
    //     println!("{:?}", program)
    // }
}
