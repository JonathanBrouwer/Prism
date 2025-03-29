// use serde::{Deserialize, Serialize};
// use std::borrow::Cow;
// use std::fmt::{Display, Formatter};
// use std::str::Chars;
//
// #[derive(Debug, Copy, Clone, Hash, Serialize, Deserialize, Eq, PartialEq)]
// pub struct EscapedString<'arn>(&'arn str);
//
// impl<'arn> EscapedString<'arn> {
//     pub fn from_escaped(s: &'arn str) -> Self {
//         Self(s)
//     }
//
//     pub fn to_cow(&self) -> Cow<'arn, str> {
//         if self.0.contains('\\') {
//             Cow::Owned(self.chars().collect())
//         } else {
//             Cow::Borrowed(self.0)
//         }
//     }
//
//     pub fn chars(&self) -> impl Iterator<Item = char> + '_ {
//         EscapedStringIter(self.0.chars())
//     }
// }
//
// impl Display for EscapedString<'_> {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}", self.0)
//     }
// }
//
// struct EscapedStringIter<'arn>(Chars<'arn>);
//
// impl Iterator for EscapedStringIter<'_> {
//     type Item = char;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         Some(match self.0.next()? {
//             '\\' => match self.0.next()? {
//                 'n' => '\n',
//                 'r' => '\r',
//                 '\\' => '\\',
//                 '"' => '"',
//                 '\'' => '\'',
//                 _ => panic!("Invalid escape sequence"),
//             },
//             c => c,
//         })
//     }
// }
