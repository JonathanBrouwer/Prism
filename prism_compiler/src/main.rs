use clap::Parser;
use prism_compiler::lang::PrismDb;
use std::process::exit;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Specifies the path to an input .pr file. If None, it means stdin is used for input.
    input: String,
}

fn main() {
    let args = Args::parse();

    let mut env = PrismDb::new();

    //Load file
    let program = env.load_file(args.input.into());
    let processed = env.process_file(program);

    // Print info
    // println!(
    //     "> Parsed Program\n====================\n{}\n\n",
    //     env.parse_index_to_string(processed.parsed),
    // );
    // println!(
    //     "> Core Program\n====================\n{}\n\n",
    //     env.index_to_string(processed.core),
    // );

    if !env.errors.is_empty() {
        env.eprint_errors();
        exit(1);
    }

    // println!(
    //     "> Type of program\n====================\n{}\n\n",
    //     env.index_to_br_string(processed.typ, &DbEnv::default())
    // );
    //
    // println!(
    //     "> Evaluated\n====================\n{}\n\n",
    //     env.index_to_br_string(processed.core, &DbEnv::default())
    // );
    exit(0);
}
