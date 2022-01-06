use std::hash::Hash;
use crate::lambday::lambday::LambdayTerm::*;

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum LambdayTerm<M, Sym: Eq + Hash + Clone> {
    Var(M, Sym),
    TypeType(M),

    FunType(M, Box<Self>, Box<Self>),
    FunConstr(M, Sym, Box<Self>, Box<Self>),
    FunDestr(M, Box<Self>, Box<Self>),

    ProdType(M, Vec<Self>),
    ProdConstr(M, Box<Self>, Vec<Self>),
    ProdDestr(M, Box<Self>, usize),

    SumType(M, Vec<Self>),
    SumConstr(M, Box<Self>, usize, Box<Self>),
    SumDestr(M, Box<Self>, Box<Self>, Vec<Self>),
}

impl<M, Sym: Eq+Hash+Clone> LambdayTerm<M, Sym> {
    pub fn meta(&self) -> &M {
        match self {
            Var(span, _) => span,
            TypeType(span) =>  span,
            FunType(span, _, _) =>  span,
            FunConstr(span, _, _, _) => span,
            FunDestr(span, _, _) =>  span,
            ProdType(span, _) =>  span,
            ProdConstr(span, _, _) =>  span,
            ProdDestr(span, _, _) =>  span,
            SumType(span, _) => span,
            SumConstr(span, _, _, _) => span,
            SumDestr(span, _, _, _) => span,
        }
    }
}

//     fn normalize_head(&self) -> Self {
//         match self {
//             Var(_, _) => self.clone(),
//             TypeType(_) => self.clone(),
//             FunType(_, _, _) => self.clone(),
//             FunConstr(_, _, _, _) => self.clone(),
//             FunDestr(span, fun, arg) => {
//                 let fun = fun.normalize_head();
//                 match &fun {
//                     FunConstr(_, sym, _arg_type, body) => {
//                         body.substitute(sym, arg).normalize_head()
//                     }
//                     _ => FunDestr(*span, Box::new(fun.clone()), arg.clone()),
//                 }
//             }
//             ProdType(_, _) => self.clone(),
//             ProdConstr(_, _, _) => self.clone(),
//             ProdDestr(span, val, num) => {
//                 let val = val.normalize_head();
//                 match &val {
//                     ProdConstr(_, _typ, vals) => {
//                         vals[*num].normalize_head()
//                     },
//                     _ => ProdDestr(*span,Box::new(val.clone()), *num)
//                 }
//             }
//             SumType(_, _) => self.clone(),
//             SumConstr(_, _, _, _) => self.clone(),
//             SumDestr(span, val, into_type, opts) => {
//                 let val = val.normalize_head();
//                 let into_type = into_type.normalize_head();
//                 match &val {
//                     SumConstr(_, _typ, num, val) => {
//                         FunDestr(*span, opts[*num].clone(), val.clone()).normalize_head()
//                     },
//                     _ => SumDestr(*span,Box::new(val.clone()), Box::new(into_type.normalize_head()), opts.clone())
//                 }
//             }
//         }
//     }

