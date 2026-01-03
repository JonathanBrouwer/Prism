use prism_compiler::lang::error::SerializedErrors;
use prism_diag::RenderConfig;
use std::cmp::max;
use std::fmt::Write;
use ui_test::diagnostics::{Diagnostics, Level, Message};
use ui_test::{
    Args, CommandBuilder, Config, default_any_file_filter, run_tests_generic, spanned,
    status_emitter,
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
                // "--release".into(),
                "--bin".into(),
                "prism_compiler".into(),
                "--".into(),
                "--error-format".into(),
                "json".into(),
            ],
            out_dir_flag: None,
            input_file_flag: None,
            envs: vec![("RUSTFLAGS".into(), Some("-Awarnings".into()))],
            cfg_flag: None,
        },
        bless_command: Some("cargo uibless".into()),
        diagnostic_extractor: |_path, output| {
            if output.is_empty() {
                return Diagnostics::default();
            }

            let errors: SerializedErrors =
                serde_json::from_slice(output).expect("Cannot parse error jsons");
            let mut rendered = String::new();

            let mut messages: Vec<Vec<Message>> = vec![];
            let input_table = errors.input_table.inner();
            for diag in errors.errors {
                let span = diag.groups[0].annotations[0].span;
                let (line, _col) = input_table.line_col_of(span.start_pos());
                messages.resize_with(max(messages.len(), line + 2), Vec::new);
                messages[line + 1].push(Message {
                    level: Level::Error,
                    message: "".to_string(),
                    line: Some(line + 1),
                    span: Some(spanned::Span {
                        file: input_table.get_path(span.start_pos().file()).to_path_buf(),
                        bytes: span.start_pos().idx_in_file()..span.end_pos().idx_in_file(),
                    }),
                    code: Some(diag.id.clone()),
                });

                writeln!(
                    &mut rendered,
                    "{}\n",
                    diag.render(&RenderConfig::uitest(), &input_table)
                )
                .unwrap();
            }

            Diagnostics {
                rendered: rendered.into(),
                messages,
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
