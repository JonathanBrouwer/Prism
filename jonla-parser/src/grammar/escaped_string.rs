use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use itertools::Itertools;

#[derive(Debug, Clone, Hash, Serialize, Deserialize, Eq, PartialEq)]
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
        self.0.chars().batching(|it| {
            let c = it.next()?;
            if c != '\\' {
                return Some(c);
            }
            Some(match it.next()? {
                'n' => '\n',
                'r' => '\r',
                '\\' => '\\',
                '"' => '"',
                '\'' => '\'',
                _ => panic!("Invalid escape sequence"),
            })
        })
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
