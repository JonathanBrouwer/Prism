#[derive(Debug, Copy, Clone, Eq)]
pub struct Input<'src> {
    file_path: &'src str,

    full_input: &'src str,
    remaining_input: &'src str,
}

impl<'src> Input<'src> {
    #[must_use]
    pub fn file_path(&self) -> &'src str {
        self.file_path
    }

    #[must_use]
    pub fn full_input(&self) -> &'src str {
        self.full_input
    }

    #[must_use]
    pub fn position(&self) -> usize {
        self.full_input.len() - self.remaining_input.len()
    }

    #[must_use]
    pub fn position_end(&self) -> usize {
        self.full_input.len()
    }

    #[must_use]
    pub fn peek(&self) -> Option<char> {
        self.remaining_input.chars().next()
    }

    #[must_use]
    pub fn max(self, other: Self) -> Self {
        assert_eq!(self.file_path, other.file_path);
        if self.position() > other.position() {
            self
        } else {
            other
        }
    }

    #[must_use]
    pub fn skip_n_chars(self, n: usize) -> Option<Self> {
        let mut chars = self.remaining_input.chars();
        for _ in 0..n {
            if chars.next().is_none() {
                return None;
            }
        }
        Some(Self {
            file_path: self.file_path,
            full_input: self.full_input,
            remaining_input: chars.as_str(),
        })
    }
}

impl<'src> PartialEq<Self> for Input<'src> {
    fn eq(&self, other: &Input<'src>) -> bool {
        self.file_path == other.file_path && self.position() == other.position()
    }
}
