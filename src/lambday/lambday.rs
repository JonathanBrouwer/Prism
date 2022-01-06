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

//     fn is_type_type(&self) -> bool {
//         if let TypeType(_) = self {
//             true
//         } else {
//             false
//         }
//     }
//
//     pub fn type_eq(&self, other: &Self) -> bool {
//         match (self.normalize_head(), other.normalize_head()) {
//             (Var(_, l1), Var(_, l2)) => l1 == l2,
//             (TypeType(_), TypeType(_)) => true,
//             (FunType(_, a1, b1), FunType(_, a2, b2)) => a1.type_eq(&a2) && b1.type_eq(&b2),
//             (ProdType(_, vs1), ProdType(_, vs2)) => vs1 == vs2,
//             (SumType(_, vs1), SumType(_, vs2)) => vs1 == vs2,
//             (_, _) => unreachable!()
//         }
//     }
//
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
//
//     fn substitute(&self, name: &Sym, to: &LambdayTerm<Sym>) -> LambdayTerm<Sym> {
//         match self {
//             Var(_, sym) => {
//                 if *sym == *name { to.clone() } else { (*self).clone() }
//             }
//             TypeType(span) => {
//                 TypeType(*span)
//             }
//             FunType(span, arg_type, body_type) => {
//                 FunType(*span,Box::new(arg_type.substitute(name, to)), Box::new(body_type.substitute(name, to)))
//             }
//             FunConstr(span, sym, arg_type, body) => {
//                 if *sym == *name {
//                     FunConstr(*span,sym.clone(), Box::new(arg_type.substitute(name, to)), Box::new((**body).clone()))
//                 } else {
//                     FunConstr(*span, sym.clone(), Box::new(arg_type.substitute(name, to)), Box::new(body.substitute(name, to)))
//                 }
//             }
//             FunDestr(span, fun, arg) => {
//                 FunDestr(*span, Box::new(fun.substitute(name, to)), Box::new(arg.substitute(name, to)))
//             }
//             ProdType(span, types) => {
//                 ProdType(*span, types.into_iter().map(|l| Box::new(l.substitute(name, to))).collect())
//             }
//             ProdConstr(span, typ, values) => {
//                 ProdConstr(*span,Box::new(typ.substitute(name, to)), values.into_iter().map(|l| Box::new(l.substitute(name, to))).collect())
//             }
//             ProdDestr(span, val, num) => {
//                 ProdDestr(*span, Box::new(val.substitute(name, to)), *num)
//             }
//             SumType(span, types) => {
//                 SumType(*span, types.into_iter().map(|l| Box::new(l.substitute(name, to))).collect())
//             }
//             SumConstr(span, typ, num, val) => {
//                 SumConstr(*span,Box::new(typ.substitute(name, to)), *num, Box::new(val.substitute(name, to)))
//             }
//             SumDestr(span, val, into_type, options) => {
//                 SumDestr(*span,Box::new(val.substitute(name, to)), Box::new(into_type.substitute(name, to)), options.into_iter().map(|l| Box::new(l.substitute(name, to))).collect())
//             }
//         }
//     }
// }

