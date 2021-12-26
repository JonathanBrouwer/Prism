use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;
use Term::*;

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum Term<Sym: Eq + Hash + Clone> {
    Var(Sym),
    TypeType(),

    FunType(Rc<Self>, Rc<Self>),
    FunConstr(Sym, Rc<Self>, Rc<Self>),
    FunDestr(Rc<Self>, Rc<Self>),
}

impl<Sym: Eq + Hash + Clone> Term<Sym> {
    fn type_check(&self, names: &mut HashMap<Sym, Term<Sym>>) -> Result<Term<Sym>, ()> {
        match self {
            Var(var) => names.get(var).map(|t| Ok(t.clone())).unwrap_or(Err(())),
            TypeType() => Ok(TypeType()), //TODO inconsistent
            FunType(arg_type, body_type) => {
                //Check if types are well formed
                let att = arg_type.type_check(names)?;
                if att != TypeType() { return Err(()); }
                let abt = body_type.type_check(names)?;
                if abt != TypeType() { return Err(()); }

                //Type of type
                Ok(TypeType())
            }
            FunConstr(sym, arg_type, body) => {
                //Check if types are well formed
                let att = arg_type.type_check(names)?;
                if att != TypeType() { return Err(()); }

                //Calc body type
                names.insert(sym.clone(), (**arg_type).clone());
                let body_type = body.type_check(names)?;
                names.remove(&sym);

                //Function type
                Ok(FunType(Rc::new((**arg_type).clone()), Rc::new(body_type)))
            }
            FunDestr(fun, arg) => {
                let fun_type = fun.type_check(names)?;
                let arg_type1 = arg.type_check(names)?;
                return if let FunType(arg_type2, body_type) = fun_type {
                    if !arg_type1.type_eq(&arg_type2) {
                        return Err(());
                    }
                    Ok((*body_type).clone())
                } else {
                    Err(())
                };
            }
        }
    }

    fn substitute(&self, name: &Sym, to: &Term<Sym>) -> Term<Sym> {
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
                        body.substitute(sym, arg)
                    }
                    _ => FunDestr(Rc::new(fun.clone()), arg.clone()),
                }
            }
        }
    }

    pub fn type_eq(&self, other: &Self) -> bool {
        match (self.normalize_head(), other.normalize_head()) {
            (Var(l1), Var(l2)) => l1 == l2,
            (TypeType(), TypeType()) => true,
            (FunType(a1, b1), FunType(a2, b2)) => a1.type_eq(&a2) && b1.type_eq(&b2),
            (_, _) => unreachable!()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::rc::Rc;
    use crate::lambday::lambday::Term::{FunConstr, FunDestr, FunType, TypeType, Var};

    #[test]
    fn test_fun_type1() {
        let term = FunConstr("a", Rc::new(TypeType()), Rc::new(Var("a")));
        let typ = FunType(Rc::new(TypeType()), Rc::new(TypeType()));
        assert_eq!(typ, term.type_check(&mut HashMap::new()).unwrap());
    }

    #[test]
    fn test_fun_type2() {
        let term = FunDestr(Rc::new(FunConstr("a", Rc::new(TypeType()), Rc::new(Var("a")))), Rc::new(TypeType()));
        let typ = TypeType();
        assert_eq!(typ, term.type_check(&mut HashMap::new()).unwrap());
    }
}


