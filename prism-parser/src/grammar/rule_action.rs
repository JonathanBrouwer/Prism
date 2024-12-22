use crate::grammar::escaped_string::EscapedString;
use crate::grammar::serde_leak::*;
use crate::parsable::parsed::Parsed;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum RuleAction<'arn, 'grm> {
    Name(&'grm str),
    InputLiteral(EscapedString<'grm>),
    Construct(
        &'grm str,
        &'grm str,
        #[serde(with = "leak_slice")] &'arn [Self],
    ),
    #[serde(skip)]
    Value(Parsed<'arn, 'grm>),
}
//
// impl<'arn, 'grm> Debug for RuleAction<'arn, 'grm> {
//     #[inline]
//     fn fmt(&self, f: &mut Formatter) -> fmt::Result {
//         match self {
//             RuleAction::Name(name) => write!(f, "{name}"),
//             RuleAction::Name(name) => write!(f, "{name}"),
//
//
//             {
//
//
//                 ::core::fmt::Formatter::debug_tuple_field1_finish(
//                     f,
//                     "Name",
//                     &__self_0,
//                 )
//             }
//             RuleAction::InputLiteral(__self_0) => {
//                 ::core::fmt::Formatter::debug_tuple_field1_finish(
//                     f,
//                     "InputLiteral",
//                     &__self_0,
//                 )
//             }
//             RuleAction::Construct(__self_0, __self_1, __self_2) => {
//                 ::core::fmt::Formatter::debug_tuple_field3_finish(
//                     f,
//                     "Construct",
//                     __self_0,
//                     __self_1,
//                     &__self_2,
//                 )
//             }
//             RuleAction::ActionResult(__self_0) => {
//                 ::core::fmt::Formatter::debug_tuple_field1_finish(
//                     f,
//                     "ActionResult",
//                     &__self_0,
//                 )
//             }
//         }
//     }
// }
