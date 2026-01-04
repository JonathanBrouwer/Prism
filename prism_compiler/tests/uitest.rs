use clap::Parser;
use libtest_mimic::{Arguments, Failed, Trial};
use prism_compiler::lang::env::DbEnv;
use prism_compiler::lang::{CoreIndex, PrismDb};
use prism_diag::RenderConfig;
use std::collections::VecDeque;
use std::convert::Into;
use std::env::args;
use std::fmt::Write;
use std::iter::Iterator;
use std::mem;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug, Clone, Default)]
pub struct FullArguments {
    #[command(flatten)]
    uitest: UitestArguments,

    #[command(flatten)]
    libtest: Arguments,
}

#[derive(Parser, Debug, Clone, Default)]
pub struct UitestArguments {
    #[arg(long)]
    bless: bool,
}

fn run_uitest(file_path: &Path, args: &UitestArguments) -> Result<(), Failed> {
    let mut env = PrismDb::default();

    let input = env.load_file(file_path.into());
    let (input, _) = env.parse_prism_file(input);
    let typ = env.type_check(input);

    // Compare stderr
    let mut stderr = String::new();
    for diag in mem::take(&mut env.diags) {
        writeln!(
            &mut stderr,
            "{}\n",
            diag.render(&RenderConfig::uitest(), &env.input.inner())
        )
        .unwrap();
    }
    compare_output(
        file_path,
        stderr.as_bytes(),
        |exp| exp == stderr.as_bytes(),
        "stderr",
        args,
    )?;
    if !stderr.is_empty() {
        return Ok(());
    }

    compare_term(file_path, &mut env, typ, "type", args)?;

    let eval = env.beta_reduce(input, &DbEnv::default());
    compare_term(file_path, &mut env, eval, "eval", args)?;

    Ok(())
}

fn compare_term(
    file_path: &Path,
    env: &mut PrismDb,
    term: CoreIndex,
    output_ext: &'static str,
    args: &UitestArguments,
) -> Result<(), String> {
    let term_str = env.index_to_sm_string(term);
    compare_output(
        file_path,
        term_str.as_bytes(),
        |expected| {
            let expected = env.load_input(
                String::from_utf8_lossy(expected).to_string(),
                file_path.with_added_extension(output_ext),
            );
            let (expected, _) = env.parse_prism_file(expected);
            env.assert_no_errors();
            env.is_beta_equal(term, &DbEnv::default(), expected, &DbEnv::default())
        },
        output_ext,
        args,
    )
}

fn compare_output(
    file_path: &Path,
    output: &[u8],
    compare_with: impl FnOnce(&[u8]) -> bool,
    output_ext: &'static str,
    args: &UitestArguments,
) -> Result<(), String> {
    let results_dir = file_path.parent().unwrap().join("_results");
    std::fs::create_dir_all(&results_dir).unwrap();
    let output_path = results_dir
        .join(file_path.file_name().unwrap())
        .with_added_extension(output_ext);

    let expected_output = if output_path.exists() {
        std::fs::read(&output_path).unwrap()
    } else {
        vec![]
    };
    if compare_with(&expected_output) {
        return Ok(());
    } else if args.bless {
        if output.is_empty() {
            _ = std::fs::remove_file(&output_path);
        } else {
            std::fs::write(&output_path, output).unwrap();
        }
        return Ok(());
    }

    if expected_output.is_empty() {
        return Err(format!(
            "Expected no `{output_ext}`, but found:\n\n{}",
            String::from_utf8_lossy(output)
        ));
    }
    if output.is_empty() {
        return Err(format!("Expected `{output_ext}`, got no output"));
    }

    Err(format!(
        "The `{output_ext}` differed. \n\n-- Expected:\n{}\n-- Actual:\n{}",
        String::from_utf8_lossy(&expected_output),
        String::from_utf8_lossy(output)
    ))
}

fn main() {
    let args = args().filter(|a| a != "--");
    let args: FullArguments = Parser::parse_from(args);

    let mut tests = vec![];

    let uitest_dir = PathBuf::from("./uitests");
    let mut queue = VecDeque::from([uitest_dir.clone()]);
    while let Some(next_dir) = queue.pop_front() {
        for item in std::fs::read_dir(&next_dir).unwrap().map(|i| i.unwrap()) {
            let abs_path = item.path();
            let item_meta = item.metadata().unwrap();

            if item_meta.is_dir() {
                queue.push_front(abs_path);
                continue;
            }

            if abs_path.extension().unwrap() != "pr" {
                continue;
            }

            let rel_path = abs_path.strip_prefix(&uitest_dir).unwrap();
            let test_name = rel_path
                .components()
                .map(|c| c.as_os_str().to_str().unwrap())
                .collect::<Vec<_>>()
                .join("::");
            let uitest_args = args.uitest.clone();
            tests.push(Trial::test(test_name, move || {
                run_uitest(&abs_path, &uitest_args)
            }))
        }
    }

    libtest_mimic::run(&args.libtest, tests).exit();
}

#[test]
fn empty() {}
