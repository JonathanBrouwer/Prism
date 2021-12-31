use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;
use itertools::Itertools;
use LambdayTerm::*;

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum LambdayTerm<Sym: Eq + Hash + Clone> {
    Var(Sym),
    TypeType(),

    FunType(Rc<Self>, Rc<Self>),
    FunConstr(Sym, Rc<Self>, Rc<Self>),
    FunDestr(Rc<Self>, Rc<Self>),

    ProdType(Vec<Rc<Self>>),
    ProdConstr(Rc<Self>, Vec<Rc<Self>>),
    ProdDestr(Rc<Self>, usize),

    SumType(Vec<Rc<Self>>),
    SumConstr(Rc<Self>, usize, Rc<Self>),
    SumDestr(Rc<Self>, Rc<Self>, Vec<Rc<Self>>)
}

impl<Sym: Eq + Hash + Clone> LambdayTerm<Sym> {
    pub fn type_check(&self) -> Result<LambdayTerm<Sym>, ()> {
        self.type_check_internal(&mut HashMap::new())
    }

    fn type_check_internal(&self, names: &mut HashMap<Sym, LambdayTerm<Sym>>) -> Result<LambdayTerm<Sym>, ()> {
        match self {
            Var(var) => names.get(var).map(|t| Ok(t.clone())).unwrap_or(Err(())),
            TypeType() => Ok(TypeType()), //TODO inconsistent
            FunType(arg_type, body_type) => {
                //Check if types are well formed
                let att = arg_type.type_check_internal(names)?;
                if att != TypeType() { return Err(()); }
                let abt = body_type.type_check_internal(names)?;
                if abt != TypeType() { return Err(()); }

                //Type of type
                Ok(TypeType())
            }
            FunConstr(sym, arg_type, body) => {
                //Check if types are well formed
                let att = arg_type.type_check_internal(names)?;
                if att != TypeType() { return Err(()); }

                //Calc body type
                names.insert(sym.clone(), (**arg_type).clone());
                let body_type = body.type_check_internal(names)?;
                names.remove(&sym);

                //Function type
                Ok(FunType(Rc::new((**arg_type).clone()), Rc::new(body_type)))
            }
            FunDestr(fun, arg) => {
                let fun_type = fun.type_check_internal(names)?;
                let arg_type1 = arg.type_check_internal(names)?;
                return if let FunType(arg_type2, body_type) = fun_type {
                    if !arg_type1.type_eq(&arg_type2) {
                        return Err(());
                    }
                    Ok((*body_type).clone())
                } else {
                    Err(())
                };
            }
            ProdType(subs) => {
                for sub in subs {
                    let sub_type = sub.type_check()?;
                    if sub_type != TypeType() { return Err(()); }
                }
                Ok(TypeType())
            }
            ProdConstr(typ, values) => {
                typ.type_check()?;
                match typ.normalize_head() {
                    ProdType(subs) => {
                        if values.len() != subs.len() { return Err(()) }
                        for (val, sub) in values.into_iter().zip_eq(subs.into_iter()) {
                            if !(val.type_check()?).type_eq(&*sub) { return Err(()) }
                        }
                        return Ok((**typ).clone())
                    }
                    _ => Err(())
                }
            }
            ProdDestr(val, num) => {
                match val.type_check()? {
                    ProdType(types) => {
                        if *num >= types.len() { return Err(()) }
                        Ok((*types[*num]).clone())
                    }
                    _ => Err(())
                }
            }
            SumType(subs) => {
                for sub in subs {
                    let sub_type = sub.type_check()?;
                    if sub_type != TypeType() { return Err(()); }
                }
                Ok(TypeType())
            }
            SumConstr(typ, num, val) => {
                typ.type_check()?;
                match typ.normalize_head() {
                    SumType(subs) => {
                        if *num >= subs.len() { return Err(()) }
                        if !(val.type_check()?).type_eq(&*subs[*num]) { return Err(()) }
                        return Ok((**typ).clone())
                    }
                    _ => Err(())
                }
            }
            SumDestr(val, into_type, opts) => {
                let val_type = val.type_check()?;
                let into_tt = into_type.type_check()?;
                if into_tt != TypeType() { return Err(()) }
                match val_type.normalize_head() {
                    SumType(subs) => {
                        if opts.len() != subs.len() { return Err(()) }

                        for (opt, sub) in opts.into_iter().zip_eq(subs.into_iter()) {
                            let opt_type = opt.type_check()?;
                            let exp = FunType(sub, into_type.clone());
                            if !exp.type_eq(&opt_type) { return Err(()) }
                        }
                        return Ok((**into_type).clone())
                    }
                    _ => Err(())
                }
            }
        }
    }

    pub fn type_eq(&self, other: &Self) -> bool {
        match (self.normalize_head(), other.normalize_head()) {
            (Var(l1), Var(l2)) => l1 == l2,
            (TypeType(), TypeType()) => true,
            (FunType(a1, b1), FunType(a2, b2)) => a1.type_eq(&a2) && b1.type_eq(&b2),
            (ProdType(vs1), ProdType(vs2)) => vs1 == vs2,
            (SumType(vs1), SumType(vs2)) => vs1 == vs2,
            (_, _) => unreachable!()
        }
    }

    fn normalize_head(&self) -> Self {
        match self {
            Var(_) => self.clone(),
            TypeType() => self.clone(),
            FunType(_, _) => self.clone(),
            FunConstr(_, _, _) => self.clone(),
            FunDestr(fun, arg) => {
                let fun = fun.normalize_head();
                match &fun {
                    FunConstr(sym, _arg_type, body) => {
                        body.substitute(sym, arg).normalize_head()
                    }
                    _ => FunDestr(Rc::new(fun.clone()), arg.clone()),
                }
            }
            ProdType(_) => self.clone(),
            ProdConstr(_, _) => self.clone(),
            ProdDestr(val, num) => {
                let val = val.normalize_head();
                match &val {
                    ProdConstr(_typ, vals) => {
                        vals[*num].normalize_head()
                    },
                    _ => ProdDestr(Rc::new(val.clone()), *num)
                }
            }
            SumType(_) => self.clone(),
            SumConstr(_, _, _) => self.clone(),
            SumDestr(val, into_type, opts) => {
                let val = val.normalize_head();
                let into_type = into_type.normalize_head();
                match &val {
                    SumConstr(_typ, num, val) => {
                        FunDestr(opts[*num].clone(), val.clone()).normalize_head()
                    },
                    _ => SumDestr(Rc::new(val.clone()), Rc::new(into_type.normalize_head()), opts.clone())
                }
            }
        }
    }

    fn substitute(&self, name: &Sym, to: &LambdayTerm<Sym>) -> LambdayTerm<Sym> {
        match self {
            Var(sym) => {
                if *sym == *name { to.clone() } else { (*self).clone() }
            }
            TypeType() => {
                TypeType()
            }
            FunType(arg_type, body_type) => {
                FunType(Rc::new(arg_type.substitute(name, to)), Rc::new(body_type.substitute(name, to)))
            }
            FunConstr(sym, arg_type, body) => {
                if *sym == *name {
                    FunConstr(sym.clone(), Rc::new(arg_type.substitute(name, to)), Rc::new((**body).clone()))
                } else {
                    FunConstr(sym.clone(), Rc::new(arg_type.substitute(name, to)), Rc::new(body.substitute(name, to)))
                }
            }
            FunDestr(fun, arg) => {
                FunDestr(Rc::new(fun.substitute(name, to)), Rc::new(arg.substitute(name, to)))
            }
            ProdType(types) => {
                ProdType(types.into_iter().map(|l| Rc::new(l.substitute(name, to))).collect())
            }
            ProdConstr(typ, values) => {
                ProdConstr(Rc::new(typ.substitute(name, to)), values.into_iter().map(|l| Rc::new(l.substitute(name, to))).collect())
            }
            ProdDestr(val, num) => {
                ProdDestr(Rc::new(val.substitute(name, to)), *num)
            }
            SumType(types) => {
                SumType(types.into_iter().map(|l| Rc::new(l.substitute(name, to))).collect())
            }
            SumConstr(typ, num, val) => {
                SumConstr(Rc::new(typ.substitute(name, to)), *num, Rc::new(val.substitute(name, to)))
            }
            SumDestr(val, into_type, options) => {
                SumDestr(Rc::new(val.substitute(name, to)), Rc::new(into_type.substitute(name, to)), options.into_iter().map(|l| Rc::new(l.substitute(name, to))).collect())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::rc::Rc;
    use crate::lambday::lambday::LambdayTerm::{FunConstr, FunDestr, FunType, TypeType, Var};

    #[test]
    fn test_fun_type1() {
        let term = FunConstr("a", Rc::new(TypeType()), Rc::new(Var("a")));
        let typ = FunType(Rc::new(TypeType()), Rc::new(TypeType()));
        assert_eq!(typ, term.type_check().unwrap());
    }

    #[test]
    fn test_fun_type2() {
        let term = FunDestr(Rc::new(FunConstr("a", Rc::new(TypeType()), Rc::new(Var("a")))), Rc::new(TypeType()));
        let typ = TypeType();
        assert_eq!(typ, term.type_check().unwrap());
    }
}


