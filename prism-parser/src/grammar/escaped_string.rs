use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use std::str::{Chars, FromStr};

#[derive(Debug, Copy, Clone, Hash, Serialize, Deserialize, Eq, PartialEq)]
pub struct EscapedString<'grm>(&'grm str);

impl<'grm> EscapedString<'grm> {
    pub fn from_escaped(s: &'grm str) -> Self {
        Self(s)
    }

    pub fn to_cow(&self) -> Cow<'grm, str> {
        if self.0.contains('\\') {
            Cow::Owned(self.chars().collect())
        } else {
            Cow::Borrowed(self.0)
        }
    }

    pub fn chars(&self) -> impl Iterator<Item = char> + '_ {
        EscapedStringIter(self.0.chars())
    }

    pub fn parse<F: FromStr>(&self) -> Result<F, F::Err> {
        self.0.parse()
    }
}

impl Display for EscapedString<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

struct EscapedStringIter<'grm>(Chars<'grm>);

impl Iterator for EscapedStringIter<'_> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        Some(match self.0.next()? {
            '\\' => match self.0.next()? {
                'n' => '\n',
                'r' => '\r',
                '\\' => '\\',
                '"' => '"',
                '\'' => '\'',
                _ => panic!("Invalid escape sequence"),
            },
            c => c,
        })
    }
}
