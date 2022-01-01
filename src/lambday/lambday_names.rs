// use std::collections::HashMap;
// use std::hash::Hash;
// use crate::jonla::jerror::JError;
// use crate::lambday::lambday::LambdayTerm;
// use crate::lambday::lambday::LambdayTerm::*;
// use crate::peg::input::Input;
//
// impl<Sym: Eq + Hash + Clone> LambdayTerm<Sym> {
//     pub fn transform_names(self) -> Result<(LambdayTerm<usize>, HashMap<usize, Sym>), JError> {
//         struct Meta<Sym: Eq + Hash + Clone> {
//             new_names: HashMap<usize, Sym>,
//             new_names_rev: HashMap<Sym, usize>,
//             errors: Vec<JError>
//         }
//         fn transform_sub<Sym: Eq + Hash + Clone>(term: LambdayTerm<Sym>, meta: &mut Meta<Sym>) -> LambdayTerm<usize> {
//             let t = |l: Box<LambdayTerm<Sym>>|
//                 box transform_sub(*l, meta);
//             let ts = |l: Vec<LambdayTerm<Sym>>|
//                 l.into_iter().map(|v| transform_sub(v, meta)).collect();
//             match term {
//                 FunConstr(span, t1, t2, t3) => {
//                     let t2 = t(t2);
//                     let id = meta.new_names.len();
//                     meta.new_names.insert(id, t1.clone());
//                     meta.new_names_rev.insert(t1, id);
//                     FunConstr(span, id, t2, t(t3))
//                 }
//                 Var(span, name) => {
//                     match meta.new_names_rev.get(&name) {
//                         None => {
//                             meta.errors.push(JError {
//                                 errors: vec![]
//                             })
//                         }
//                         Some(nid) => {
//                             Var(span, *nid)
//                         }
//                     }
//                 }
//                 TypeType(span) =>
//                     TypeType(span),
//                 FunType(span, t1, t2) =>
//                     FunType(span, t(t1), t(t2)),
//                 FunDestr(span, t1, t2) =>
//                     FunDestr(span, t(t1), t(t2)),
//                 ProdType(span, t1) =>
//                     ProdType(span, ts(t1)),
//                 ProdConstr(span, t1, t2) =>
//                     ProdConstr(span, t(t1), ts(t2)),
//                 ProdDestr(span, t1, t2) =>
//                     ProdDestr(span, t(t1), t2),
//                 SumType(span, t1) =>
//                     SumType(span, ts(t1)),
//                 SumConstr(span, t1, t2, t3) =>
//                     SumConstr(span, t(t1), t2, t(t3)),
//                 SumDestr(span, t1, t2, t3) =>
//                     SumDestr(span, t(t1), t(t2), ts(t3))
//             }
//         };
//         // let mut hm = HashMap::new();
//         // let term = transform_sub(self, &mut hm, &mut HashMap::new());
//         // Ok((term, hm))
//         todo!()
//     }
// }