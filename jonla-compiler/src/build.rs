use jonla_macros::handle_language;
use std::path::PathBuf;

fn main() {
    let path: PathBuf = "resources/grammar".into();
    handle_language(path);
}
