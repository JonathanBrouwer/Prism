use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Like a file, formats contents on closing
pub struct FormattingFile(Option<File>, PathBuf, bool);

impl FormattingFile {
    pub fn create_formatting(p: impl AsRef<Path>) -> io::Result<Self> {
        Ok(Self(Some(File::create(&p)?), p.as_ref().to_path_buf(), true))
    }
    pub fn create_not_formatting(p: impl AsRef<Path>) -> io::Result<Self> {
        Ok(Self(Some(File::create(&p)?), p.as_ref().to_path_buf(), false))
    }
}

fn try_fmt(p: impl AsRef<Path>) -> io::Result<()> {
    Command::new("rustfmt").arg(p.as_ref()).spawn()?.wait()?;

    Ok(())
}

impl Drop for FormattingFile {
    fn drop(&mut self) {
        drop(self.0.take());

        if self.2 {
            if let Err(e) = try_fmt(&self.1) {
                eprintln!("{}", e);
            }
        }
    }
}

impl Read for FormattingFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.as_ref().unwrap().read(buf)
    }
}

impl Write for FormattingFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.as_ref().unwrap().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.as_ref().unwrap().flush()
    }
}
