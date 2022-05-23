// use crate::term::Term;
// use crate::type_check::TypeCheckError;
// use std::rc::Rc;
//
// pub type Env<'a> = Option<Rc<EnvBody<'a>>>;
// #[derive(Eq, PartialEq)]
// pub struct EnvBody<'a> {
//     pub parent: Env<'a>,
//     pub name: &'a str,
//     pub typ: (Term<'a>, Env<'a>),
// }
// pub trait RcEnv<'a>: Sized {
//     fn empty() -> Self;
//     fn query(&self, key: &'a str) -> Result<&EnvBody<'a>, TypeCheckError<'a>>;
//     fn extend(
//         &self,
//         name: &'a str,
//         typ: (Term<'a>, Env<'a>),
//     ) -> Env<'a>;
//     fn debug(&self) -> String;
// }
// impl<'a> RcEnv<'a> for Env<'a> {
//     fn empty() -> Self {
//         None
//     }
//
//     fn query(&self, key: &'a str) -> Result<&EnvBody<'a>, TypeCheckError<'a>> {
//         match self {
//             Some(body) if body.name == key => Ok(&body),
//             Some(body) => body.parent.query(key),
//             None => Err(TypeCheckError::NameNotFound(key)),
//         }
//     }
//
//     fn extend(
//         &self,
//         name: &'a str,
//         typ: (Term<'a>, Env<'a>),
//     ) -> Env<'a> {
//         Some(Rc::new(EnvBody {
//             parent: (*self).clone(),
//             name,
//             typ,
//         }))
//     }
//
//     fn debug(&self) -> String {
//         match self {
//             None => String::new(),
//             Some(body) => {
//                 if body.parent.is_some() {
//                     format!(
//                         "({}:{:?}) {}",
//                         body.name,
//                         body.typ.0,
//                         body.parent.debug()
//                     )
//                 } else {
//                     format!(
//                         "({}:{:?})",
//                         body.name,
//                         body.typ.0,
//                     )
//                 }
//             }
//         }
//     }
// }
