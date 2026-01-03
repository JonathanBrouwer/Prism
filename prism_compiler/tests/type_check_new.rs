use ui_test::diagnostics::Diagnostics;
use ui_test::{
    Args, CommandBuilder, Config, default_any_file_filter, run_tests_generic, status_emitter,
};

#[cfg(test)]
fn main() -> ui_test::color_eyre::Result<()> {
    let args = Args::test()?;
    let mut config = Config {
        host: Some("host".to_string()),
        root_dir: "./uitests".into(),
        program: CommandBuilder {
            program: "cargo".into(),
            args: vec![
                "run".into(),
                "--quiet".into(),
                "--release".into(),
                "--bin".into(),
                "prism-compiler".into(),
            ],
            out_dir_flag: None,
            input_file_flag: None,
            envs: vec![("RUSTFLAGS".into(), Some("-Awarnings".into()))],
            cfg_flag: None,
        },
        bless_command: Some("cargo uibless".into()),
        diagnostic_extractor: |path, output| {
            println!("{:?}", String::from_utf8_lossy(output));
            Diagnostics {
                rendered: vec![],
                messages: vec![],
                messages_from_unknown_file_or_line: vec![],
            }
        },
        ..Config::dummy()
    };
    config.with_args(&args);

    run_tests_generic(
        vec![config],
        |path, config| {
            path.extension().filter(|&ext| ext == "pr")?;
            Some(default_any_file_filter(path, config))
        },
        |_config, _| {},
        status_emitter::Text::verbose(),
    )
}
