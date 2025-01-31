use bumpalo::Bump;
use clap::Parser;
use prism_compiler::lang::PrismEnv;
use prism_compiler::parser::parse_prism_in_env;
use prism_parser::core::cache::Allocs;
use prism_parser::error::aggregate_error::ParseResultExt;
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

    let bump = Bump::new();
    let allocs = Allocs::new(&bump);
    let mut env = PrismEnv::new(allocs);

    // Parse
    let idx = parse_prism_in_env(&program, &mut env).unwrap_or_eprint();
    let idx = env.parsed_to_checked(idx);
    println!(
        "> Program\n====================\n{}\n\n",
        env.index_to_string(idx),
    );

    // Type check
    match env.type_check(idx) {
        Ok(i) => println!(
            "> Type of program\n====================\n{}\n\n",
            env.index_to_br_string(i)
        ),
        Err(e) => {
            e.eprint(&mut env, &program).unwrap();
            return;
        }
    }

    // Eval
    println!(
        "> Evaluated\n====================\n{}\n\n",
        env.index_to_br_string(idx)
    );
}
