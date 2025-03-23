use bumpalo::Bump;
use clap::Parser;
use prism_compiler::lang::PrismEnv;
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

    let bump = Bump::new();
    let allocs = Allocs::new(&bump);
    let mut env = PrismEnv::new(allocs);

    //Load file
    let program = env.load_file(args.input.into());

    // Parse
    let idx = env.parse_file(program).unwrap_or_eprint();
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
