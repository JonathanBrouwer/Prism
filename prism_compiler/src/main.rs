use clap::Parser;
use prism_compiler::args::PrismArgs;
use prism_compiler::lang::PrismDb;
use std::process::exit;

fn main() {
    let args = PrismArgs::parse();
    let mut env = PrismDb::new(args);

    //Load file
    let program = env.load_main_file();
    let _processed = env.process_file(program);

    // Print info
    // println!(
    //     "> Parsed Program\n====================\n{}\n\n",
    //     env.parse_index_to_string(processed.parsed),
    // );
    // println!(
    //     "> Core Program\n====================\n{}\n\n",
    //     env.index_to_string(processed.core),
    // );

    if !env.diags.is_empty() {
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
