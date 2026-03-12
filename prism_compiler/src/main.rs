use clap::Parser;
use prism_compiler::args::PrismArgs;
use prism_compiler::lang::Database;
use std::process::exit;

fn main() {
    let args = PrismArgs::parse();
    let mut env = Database::new(args);

    //Load file
    let _processed = env.process_main_file();

    let diags = env.take_diags();
    eprintln!("{diags}");
    if let Err(_) = diags.has_errored() {
        exit(1);
    }

    exit(0);
}
