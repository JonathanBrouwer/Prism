use bumpalo::Bump;
use clap::Parser;
use prism_compiler::lang::PrismEnv;
use prism_compiler::parser::parse_prism_in_env;
use prism_parser::core::allocs::Allocs;
use prism_parser::error::aggregate_error::ParseResultExt;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Specifies the path to an input .pr file. If None, it means stdin is used for input.
    input: String,
}

fn main() {
    let args = Args::parse();

    let program = std::fs::read_to_string(args.input).unwrap();

    let bump = Bump::new();
    let allocs = Allocs::new(&bump);
    let mut env = PrismEnv::new(allocs);

    // Parse
    let idx = parse_prism_in_env(&program, &mut env).unwrap_or_eprint();
    println!(
        "> Parsed Program\n====================\n{}\n\n",
        env.parse_index_to_string(idx),
    );

    let idx = env.parsed_to_checked(idx);
    println!(
        "> Core Program\n====================\n{}\n\n",
        env.index_to_string(idx),
    );

    // Type check
    match env.type_check(idx) {
        Ok(i) => println!(
            "> Type of program\n====================\n{}\n\n",
            env.index_to_br_string(i)
        ),
        Err(e) => {
            e.eprint(&mut env).unwrap();
            return;
        }
    }

    // Eval
    println!(
        "> Evaluated\n====================\n{}\n\n",
        env.index_to_br_string(idx)
    );
}
