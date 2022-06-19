use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Like a file, formats contents on closing
pub struct FormattingFile(Option<File>, PathBuf, bool);

impl FormattingFile {
    /// Create a new `FormattingFile`, that formats on close
    pub fn create_formatting(p: impl AsRef<Path>) -> io::Result<Self> {
        Ok(Self(
            Some(File::create(&p)?),
            p.as_ref().to_path_buf(),
            true,
        ))
    }

    /// Create a new `FormattingFile`, that doesn't format on close
    pub fn create_not_formatting(p: impl AsRef<Path>) -> io::Result<Self> {
        Ok(Self(
            Some(File::create(&p)?),
            p.as_ref().to_path_buf(),
            false,
        ))
    }
}

/// Try to format the file at the given path
fn try_fmt(p: impl AsRef<Path>) -> io::Result<()> {
    Command::new("rustfmt").arg(p.as_ref()).spawn()?.wait()?;

    Ok(())
}

impl Drop for FormattingFile {
    /// On drop, call try_format
    fn drop(&mut self) {
        drop(self.0.take());

        if self.2 {
            if let Err(e) = try_fmt(&self.1) {
                eprintln!("{}", e);
            }
        }
    }
}

/// Read to file
impl Read for FormattingFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.as_ref().unwrap().read(buf)
    }
}

/// Write to file
impl Write for FormattingFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.as_ref().unwrap().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.as_ref().unwrap().flush()
    }
}
