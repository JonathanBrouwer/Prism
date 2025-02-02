use ui_test::{
    Args, CommandBuilder, Config, default_any_file_filter, run_tests_generic, status_emitter,
};

#[cfg(test)]
fn main() -> ui_test::color_eyre::Result<()> {
    let args = Args::test()?;
    let mut config = Config {
        host: Some("host".to_string()),
        root_dir: "./programs_new".into(),
        program: CommandBuilder {
            program: "echo".into(),
            args: vec![],
            out_dir_flag: None,
            input_file_flag: None,
            envs: vec![],
            cfg_flag: None,
        },
        bless_command: Some("cargo uibless".into()),
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
