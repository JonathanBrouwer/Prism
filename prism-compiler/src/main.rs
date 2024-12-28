use bumpalo::Bump;
use clap::Parser;
use prism_compiler::lang::{TcEnv, UnionIndex};
use prism_compiler::parser::GRAMMAR;
use prism_parser::core::cache::Allocs;
use prism_parser::error::set_error::SetError;
use prism_parser::parsable::parsable_dyn::ParsableDyn;
use prism_parser::parser::parser_instance::run_parser_rule;
use std::collections::HashMap;
use std::io::Read;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Specifies the path to an input .pr file. If None, it means stdin is used for input.
    input: Option<String>,
}

fn read_from_stdin() -> Result<String, std::io::Error> {
    let mut program = String::new();
    std::io::stdin().read_to_string(&mut program)?;
    Ok(program)
}

fn main() {
    let args = Args::parse();

    let (program, _filename) = match args.input.as_ref() {
        None => (read_from_stdin().unwrap(), "stdin"),
        Some(file) => (std::fs::read_to_string(file).unwrap(), file.as_str()),
    };
    //
    // let bump = Bump::new();
    // let allocs = Allocs::new(&bump);
    // let mut tc_env = TcEnv::default();
    // let mut parsables = HashMap::new();
    // parsables.insert("Expr", ParsableDyn::new::<UnionIndex>());
    //
    // let root = match run_parser_rule::<UnionIndex, SetError>(
    //     &GRAMMAR, "expr", &program, allocs, parsables,
    // ) {
    //     Ok(idx) => idx,
    //     Err(e) => {
    //         e.eprint().unwrap();
    //         return;
    //     }
    // };

    // println!(
    //     "> Program\n====================\n{}\n\n",
    //     tc_env.index_to_string(root),
    // );

    // match tc_env.type_check(root) {
    //     Ok(i) => println!(
    //         "> Type of program\n====================\n{}\n\n",
    //         tc_env.index_to_br_string(i)
    //     ),
    //     Err(e) => {
    //         e.eprint(&mut tc_env, &program).unwrap();
    //         return;
    //     }
    // }
    //
    // println!(
    //     "> Evaluated\n====================\n{}\n\n",
    //     tc_env.index_to_br_string(root)
    // );
}
