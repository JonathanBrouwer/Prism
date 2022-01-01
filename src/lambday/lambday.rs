use std::hash::Hash;
use crate::jonla::jerror::Span;
use crate::lambday::lambday::LambdayTerm::*;

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum LambdayTerm<Sym: Eq + Hash + Clone> {
    Var(Span, Sym),
    TypeType(Span),

    FunType(Span, Box<Self>, Box<Self>),
    FunConstr(Span, Sym, Box<Self>, Box<Self>),
    FunDestr(Span, Box<Self>, Box<Self>),

    ProdType(Span, Vec<Self>),
    ProdConstr(Span, Box<Self>, Vec<Self>),
    ProdDestr(Span, Box<Self>, usize),

    SumType(Span, Vec<Self>),
    SumConstr(Span, Box<Self>, usize, Box<Self>),
    SumDestr(Span, Box<Self>, Box<Self>, Vec<Self>)
}

// impl<Sym: Eq + Hash + Clone> LambdayTerm<Sym> {
//     pub fn type_check(&self) -> Result<LambdayTerm<Sym>, ()> {
//         self.type_check_internal(&mut HashMap::new())
//     }
//
//     pub fn span(&self) -> Span {
//         *match self {
//             Var(span, _) => span,
//             TypeType(span) =>  span,
//             FunType(span, _, _) =>  span,
//             FunConstr(span, _, _, _) => span,
//             FunDestr(span, _, _) =>  span,
//             ProdType(span, _) =>  span,
//             ProdConstr(span, _, _) =>  span,
//             ProdDestr(span, _, _) =>  span,
//             SumType(span, _) => span,
//             SumConstr(span, _, _, _) => span,
//             SumDestr(span, _, _, _) => span,
//         }
//     }
//
//     fn check_is_type(&self, names: &mut HashMap<Sym, LambdayTerm<Sym>>) -> Result<(), ()> {
//         match self.type_check_internal(names)? {
//             TypeType(_) => Ok(()),
//             _ => Err(())
//         }
//     }
//
//     fn type_check_internal(&self, names: &mut HashMap<Sym, LambdayTerm<Sym>>) -> Result<LambdayTerm<Sym>, ()> {
//         match self {
//             Var(_span, var) => names.get(var).map(|t| Ok(t.clone())).unwrap_or(Err(())),
//             TypeType(span, ) => Ok(TypeType(*span)), //TODO inconsistent
//             FunType(span, arg_type, body_type) => {
//                 arg_type.check_is_type(names)?;
//                 body_type.check_is_type(names)?;
//                 Ok(TypeType(*span))
//             }
//             FunConstr(span, sym, arg_type, body) => {
//                 arg_type.check_is_type(names)?;
//
//                 //Calc body type
//                 names.insert(sym.clone(), (**arg_type).clone());
//                 let body_type = body.type_check_internal(names)?;
//                 names.remove(&sym);
//
//                 //Function type
//                 Ok(FunType(*span, Box::new((**arg_type).clone()), Box::new(body_type)))
//             }
//             FunDestr(_span, fun, arg) => {
//                 let fun_type = fun.type_check_internal(names)?;
//                 let arg_type1 = arg.type_check_internal(names)?;
//                 return if let FunType(_, arg_type2, body_type) = fun_type {
//                     if !arg_type1.type_eq(&arg_type2) {
//                         return Err(());
//                     }
//                     Ok((*body_type).clone())
//                 } else {
//                     Err(())
//                 };
//             }
//             ProdType(span, subs) => {
//                 for sub in subs {
//                     sub.check_is_type(names)?;
//                 }
//                 Ok(TypeType(*span))
//             }
//             ProdConstr(_, typ, values) => {
//                 typ.type_check()?;
//                 match typ.normalize_head() {
//                     ProdType(_, subs) => {
//                         if values.len() != subs.len() { return Err(()) }
//                         for (val, sub) in values.into_iter().zip_eq(subs.into_iter()) {
//                             if !(val.type_check()?).type_eq(&*sub) { return Err(()) }
//                         }
//                         return Ok((**typ).clone())
//                     }
//                     _ => Err(())
//                 }
//             }
//             ProdDestr(_, val, num) => {
//                 match val.type_check()? {
//                     ProdType(_, types) => {
//                         if *num >= types.len() { return Err(()) }
//                         Ok((*types[*num]).clone())
//                     }
//                     _ => Err(())
//                 }
//             }
//             SumType(span, subs) => {
//                 for sub in subs {
//                     sub.check_is_type(names)?;
//                 }
//                 Ok(TypeType(*span))
//             }
//             SumConstr(_, typ, num, val) => {
//                 typ.type_check()?;
//                 match typ.normalize_head() {
//                     SumType(_, subs) => {
//                         if *num >= subs.len() { return Err(()) }
//                         if !(val.type_check()?).type_eq(&*subs[*num]) { return Err(()) }
//                         return Ok((**typ).clone())
//                     }
//                     _ => Err(())
//                 }
//             }
//             SumDestr(_, val, into_type, opts) => {
//                 let val_type = val.type_check()?;
//                 let into_tt = into_type.type_check()?;
//                 if !into_tt.is_type_type() { return Err(()) }
//                 match val_type.normalize_head() {
//                     SumType(_, subs) => {
//                         if opts.len() != subs.len() { return Err(()) }
//
//                         for (opt, sub) in opts.into_iter().zip_eq(subs.into_iter()) {
//                             let opt_type = opt.type_check()?;
//                             let exp = FunType(sub.span(), sub, into_type.clone());
//                             if !exp.type_eq(&opt_type) { return Err(()) }
//                         }
//                         return Ok((**into_type).clone())
//                     }
//                     _ => Err(())
//                 }
//             }
//         }
//     }
//
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

