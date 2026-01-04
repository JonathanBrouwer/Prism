use clap::{Parser, ValueEnum};
use std::fmt::{Display, Formatter};

#[derive(ValueEnum, Debug, Copy, Clone, Default)]
pub enum ErrorFormat {
    #[default]
    Pretty,
    Plain,
}

impl Display for ErrorFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorFormat::Pretty => write!(f, "pretty"),
            ErrorFormat::Plain => write!(f, "plain"),
        }
    }
}

#[derive(Parser, Debug, Default)]
#[command(author, version, about, long_about = None)]
pub struct PrismArgs {
    #[arg(long, default_value_t)]
    pub error_format: ErrorFormat,

    /// Specifies the path to an input .pr file. If None, it means stdin is used for input.
    pub input: String,
}
