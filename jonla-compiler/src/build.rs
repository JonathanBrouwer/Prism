use std::path::PathBuf;
use jonla_macros::handle_language;

fn main() {
    let path: PathBuf = "resources/grammar".into();
    handle_language(path);
}
