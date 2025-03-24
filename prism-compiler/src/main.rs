use bumpalo::Bump;
use clap::Parser;
use prism_compiler::lang::PrismEnv;
use prism_parser::core::allocs::Allocs;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Specifies the path to an input .pr file. If None, it means stdin is used for input.
    input: String,
}

fn main() {
    let args = Args::parse();

    let bump = Bump::new();
    let allocs = Allocs::new(&bump);
    let mut env = PrismEnv::new(allocs);

    //Load file
    let program = env.load_file(args.input.into());
    let processed = env.process_file(program);

    // Print info
    println!(
        "> Parsed Program\n====================\n{}\n\n",
        env.parse_index_to_string(processed.parsed),
    );
    println!(
        "> Core Program\n====================\n{}\n\n",
        env.index_to_string(processed.core),
    );
    println!(
        "> Type of program\n====================\n{}\n\n",
        env.index_to_br_string(processed.typ)
    );

    if !env.errors.is_empty() {
        println!("> Errors\n====================\n",);
        env.eprint_errors();
    } else {
        println!(
            "> Evaluated\n====================\n{}\n\n",
            env.index_to_br_string(processed.core)
        );
    }
}
