use std::collections::HashMap;
use std::hash::Hash;
use crate::jonla::jerror::{JError, JErrorEntry};
use crate::lambday::lambday::LambdayTerm;
use crate::lambday::lambday::LambdayTerm::*;

impl<Sym: Eq + Hash + Clone> LambdayTerm<Sym> {
    pub fn transform_names(self) -> Result<(LambdayTerm<usize>, HashMap<usize, Sym>), JError> {
        struct Meta<Sym: Eq + Hash + Clone> {
            new_names: HashMap<usize, Sym>,
            new_names_rev: HashMap<Sym, usize>,
            errors: Vec<JErrorEntry>
        }
        fn transform_sub<Sym: Eq + Hash + Clone>(term: LambdayTerm<Sym>, meta: &mut Meta<Sym>) -> LambdayTerm<usize> {
            let t = |l: Box<LambdayTerm<Sym>>, meta: &mut Meta<Sym>|
                box transform_sub(*l, meta);
            let ts = |l: Vec<LambdayTerm<Sym>>, meta: &mut Meta<Sym>|
                l.into_iter().map(|v| transform_sub(v, meta)).collect();
            match term {
                FunConstr(span, t1, t2, t3) => {
                    let t2 = t(t2, meta);
                    let id = meta.new_names.len();
                    meta.new_names.insert(id, t1.clone());
                    meta.new_names_rev.insert(t1, id);
                    FunConstr(span, id, t2, t(t3, meta))
                }
                Var(span, name) => {
                    match meta.new_names_rev.get(&name) {
                        None => {
                            meta.errors.push(JErrorEntry::NameUndefined(span));
                            Var(span, usize::MAX)
                        }
                        Some(nid) => {
                            Var(span, *nid)
                        }
                    }
                }
                TypeType(span) =>
                    TypeType(span),
                FunType(span, t1, t2) =>
                    FunType(span, t(t1, meta), t(t2, meta)),
                FunDestr(span, t1, t2) =>
                    FunDestr(span, t(t1, meta), t(t2, meta)),
                ProdType(span, t1) =>
                    ProdType(span, ts(t1, meta)),
                ProdConstr(span, t1, t2) =>
                    ProdConstr(span, t(t1, meta), ts(t2, meta)),
                ProdDestr(span, t1, t2) =>
                    ProdDestr(span, t(t1, meta), t2),
                SumType(span, t1) =>
                    SumType(span, ts(t1, meta)),
                SumConstr(span, t1, t2, t3) =>
                    SumConstr(span, t(t1, meta), t2, t(t3, meta)),
                SumDestr(span, t1, t2, t3) =>
                    SumDestr(span, t(t1, meta), t(t2, meta), ts(t3, meta))
            }
        };
        let mut meta = Meta {
            new_names: HashMap::new(),
            new_names_rev: HashMap::new(),
            errors: vec![]
        };
        let term = transform_sub(self, &mut meta);
        if meta.errors.len() == 0 {
            Ok((term, meta.new_names))
        } else {
            Err(JError { errors: meta.errors })
        }
    }
}