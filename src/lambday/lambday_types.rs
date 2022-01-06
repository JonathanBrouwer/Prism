use std::collections::{HashMap, VecDeque};
use itertools::Itertools;
use crate::jonla::jerror::{JError, JErrorEntry, Span};
use crate::lambday::lambday::LambdayTerm;
use crate::lambday::lambday::LambdayTerm::*;

#[derive(Debug, Clone)]
pub struct TypeCheckMeta {
    span: Span,
    typ: Option<LambdayTerm<Span, usize>>,
}

struct TypeCheckState {
    names_types: HashMap<usize, Option<LambdayTerm<Span, usize>>>,
    names_vals: HashMap<usize, LambdayTerm<TypeCheckMeta, usize>>,
    call_stack: VecDeque<LambdayTerm<TypeCheckMeta, usize>>,
    //TODO add errors
}

impl LambdayTerm<Span, usize> {
    pub fn type_check(self) -> Result<LambdayTerm<TypeCheckMeta, usize>, JError> {
        let mut errors = Vec::new();
        let term = self.type_check_inner(&mut TypeCheckState { names_types: HashMap::new(), names_vals: HashMap::new(), call_stack: VecDeque::new() }, &mut errors);
        if errors.len() == 0 {
            Ok(term)
        } else {
            Err(JError { errors })
        }
    }

    fn type_check_inner(self, state: &mut TypeCheckState, errors: &mut Vec<JErrorEntry>) -> LambdayTerm<TypeCheckMeta, usize> {
        match self {
            Var(span, name) => {
                Var(TypeCheckMeta { span, typ: state.names_types.get(&name).unwrap().clone() }, name)
            }
            TypeType(span) => {
                TypeType(TypeCheckMeta { span, typ: Some(self) })
            }
            FunType(span, arg_type, body_type) => {
                let arg_type: LambdayTerm<TypeCheckMeta, usize> = arg_type.type_check_inner(state, errors);
                let body_type: LambdayTerm<TypeCheckMeta, usize> = body_type.type_check_inner(state, errors);
                arg_type.check_is_type(state, errors);
                body_type.check_is_type(state, errors);

                FunType(TypeCheckMeta { span, typ: Some(TypeType(span)) }, box arg_type, box body_type)
            }
            FunConstr(span, sym, arg_type, body) => {
                let arg_type_checked: LambdayTerm<TypeCheckMeta, usize> = arg_type.clone().type_check_inner(state, errors);

                //Calc body type
                if arg_type_checked.check_is_type(state, errors) {
                    state.names_types.insert(sym, Some((*arg_type).clone()));
                } else {
                    state.names_types.insert(sym, None);
                }
                let body: LambdayTerm<TypeCheckMeta, usize> = body.type_check_inner(state, errors);
                state.names_types.remove(&sym);

                //If body has a valid type return function type to body
                let typ = if body.meta().typ.is_some() {
                    Some(FunType(span, box (*arg_type).clone(), box body.meta().typ.as_ref().unwrap().clone()))
                } else {
                    None
                };
                FunConstr(TypeCheckMeta { span, typ }, sym, box arg_type_checked, box body)
            }
            FunDestr(span, fun, arg) => {
                let fun: LambdayTerm<TypeCheckMeta, usize> = fun.type_check_inner(state, errors);
                let arg: LambdayTerm<TypeCheckMeta, usize> = arg.type_check_inner(state, errors);

                if fun.meta().typ.is_none() {
                    return FunDestr(TypeCheckMeta { span, typ: None }, box fun, box arg);
                }
                return if let FunType(_, fun_arg_type, fun_body_type) = fun.meta().typ.clone().unwrap().normalize_head() {
                    arg.check_has_type(fun_arg_type.as_ref(), state, errors);

                    let typ: LambdayTerm<Span, usize> = *fun_body_type;
                    FunDestr(TypeCheckMeta { span, typ: Some(typ) }, box fun, box arg)
                } else {
                    errors.push(JErrorEntry::TypeExpectFunc(fun.meta().span));
                    FunDestr(TypeCheckMeta { span, typ: None }, box fun, box arg)
                };
            }
            ProdType(span, subtypes) => {
                let subtypes = subtypes.into_iter().map(|subtype| {
                    let subtype: LambdayTerm<TypeCheckMeta, usize> = subtype.type_check_inner(state, errors);
                    subtype.check_is_type(state, errors);
                    subtype
                }).collect();

                ProdType(TypeCheckMeta { span, typ: Some(TypeType(span)) }, subtypes)
            }
            ProdConstr(span, typ, values) => {
                let typ: Box<LambdayTerm<Span, usize>> = typ;
                let typ_checked: LambdayTerm<TypeCheckMeta, usize> = typ.clone().type_check_inner(state, errors);
                let values: Vec<LambdayTerm<TypeCheckMeta, usize>> = values.into_iter().map(|v| v.type_check_inner(state, errors)).collect();

                match typ.clone().normalize_head() {
                    ProdType(_, subtypes) => {
                        if values.len() != subtypes.len() {
                            //TODO specifically highlight too little/many arguments
                            errors.push(JErrorEntry::TypeWrongArgumentCount(span, subtypes.len(), values.len()))
                        } else {
                            for (val, sub) in values.iter().zip_eq(subtypes.iter()) {
                                val.check_has_type(&sub, state, errors);
                            }
                        }

                        ProdConstr(TypeCheckMeta { span, typ: Some((*typ).clone()) }, box typ_checked, values)
                    }
                    _ => {
                        errors.push(JErrorEntry::TypeExpectProd(*typ.meta()));
                        ProdConstr(TypeCheckMeta { span, typ: None }, box typ_checked, values)
                    }
                }
            }
            ProdDestr(span, val, num) => {
                let val: LambdayTerm<TypeCheckMeta, usize> = val.type_check_inner(state, errors);
                let typ: Option<LambdayTerm<Span, usize>> = match val.meta().typ.as_ref().map(|t| t.clone().normalize_head()) {
                    Some(ProdType(_, types)) => {
                        if num >= types.len() {
                            errors.push(JErrorEntry::TypeInvalidNumber(span));
                            None
                        } else {
                            Some(types[num].clone())
                        }
                    }
                    Some(_) => {
                        errors.push(JErrorEntry::TypeExpectProd(val.meta().span));
                        None
                    }
                    None => {
                        None
                    }
                };
                ProdDestr(TypeCheckMeta { span, typ }, box val, num)
            }
            SumType(span, subtypes) => {
                let subtypes = subtypes.into_iter().map(|subtype| {
                    let subtype: LambdayTerm<TypeCheckMeta, usize> = subtype.type_check_inner(state, errors);
                    subtype.check_is_type(state, errors);
                    subtype
                }).collect();

                SumType(TypeCheckMeta { span, typ: Some(TypeType(span)) }, subtypes)
            }
            SumConstr(span, typ, num, val) => {
                let typ: Box<LambdayTerm<Span, usize>> = typ;
                let typ_checked: LambdayTerm<TypeCheckMeta, usize> = typ.clone().type_check_inner(state, errors);
                let val: LambdayTerm<TypeCheckMeta, usize> = val.type_check_inner(state, errors);

                match typ.clone().normalize_head() {
                    SumType(_, subs) => {
                        if num >= subs.len() {
                            errors.push(JErrorEntry::TypeInvalidNumber(span))
                        }
                        val.check_has_type(&subs[num], state, errors);
                        SumConstr(TypeCheckMeta { span, typ: Some((*typ).clone()) }, box typ_checked, num, box val)
                    }
                    _ => {
                        errors.push(JErrorEntry::TypeExpectSum(*typ.meta()));
                        SumConstr(TypeCheckMeta { span, typ: None }, box typ_checked, num, box val)
                    }
                }
            }
            SumDestr(span, val, into_type, opts) => {
                let into_type_checked: LambdayTerm<TypeCheckMeta, usize> = into_type.clone().type_check_inner(state, errors);
                let val: LambdayTerm<TypeCheckMeta, usize> = val.type_check_inner(state, errors);
                let opts: Vec<LambdayTerm<TypeCheckMeta, usize>> = opts.into_iter().map(|v| v.type_check_inner(state, errors)).collect();

                if into_type_checked.meta().typ.is_none() || !into_type_checked.check_is_type(state, errors) {
                    return SumDestr(TypeCheckMeta { span, typ: None }, box val, box into_type_checked, opts);
                }

                match val.meta().typ.as_ref().map(|t| t.clone().normalize_head()) {
                    Some(SumType(_, subtypes)) => {
                        if subtypes.len() != opts.len() {
                            errors.push(JErrorEntry::TypeWrongArgumentCount(span, subtypes.len(), opts.len()))
                        } else {
                            for (val, subtype) in opts.iter().zip_eq(subtypes.into_iter()) {
                                let exp: LambdayTerm<Span, usize> = FunType(*subtype.meta(), box subtype.clone(), into_type.clone());
                                val.check_has_type(&exp, state, errors);
                            }
                        }

                        SumDestr(TypeCheckMeta { span, typ: Some((*into_type).clone()) }, box val, box into_type_checked, opts)
                    }
                    _ => {
                        errors.push(JErrorEntry::TypeExpectSum(val.meta().span));
                        SumDestr(TypeCheckMeta { span, typ: None }, box val, box into_type_checked, opts)
                    }
                }
            }
        }
    }

    fn is_type_eq(&self, other: &Self, _names: &mut TypeCheckState) -> bool {
        match (self.clone().normalize_head(), other.clone().normalize_head()) {
            (Var(_, l1), Var(_, l2)) => l1 == l2, //TODO
            (TypeType(_), TypeType(_)) => true,
            (FunType(_, a1, b1), FunType(_, a2, b2)) =>
                a1.is_type_eq(&a2, _names) && b1.is_type_eq(&b2,  _names),
            (ProdType(_, vs1), ProdType(_, vs2)) => vs1 == vs2,
            (SumType(_, vs1), SumType(_, vs2)) => vs1 == vs2,
            (_, _) => false
        }
    }
}

impl LambdayTerm<TypeCheckMeta, usize> {
    fn check_is_type(&self, _names: &mut TypeCheckState, errors: &mut Vec<JErrorEntry>) -> bool {
        let norm = self.clone().normalize_head();
        match norm.meta().typ {
            Some(TypeType(_)) => true,
            Some(_) => {
                errors.push(JErrorEntry::TypeExpectType(self.meta().span));
                false
            },
            None => false,
        }
    }

    fn check_has_type(&self, other: &LambdayTerm<Span, usize>, _names: &mut TypeCheckState, errors: &mut Vec<JErrorEntry>) -> bool {
        if self.meta().typ.is_none() { return false; }
        if self.meta().typ.as_ref().unwrap().is_type_eq(other, _names) {
            true
        } else {
            errors.push(JErrorEntry::TypeExprHasType(self.meta().span, *other.meta()));
            false
        }
    }
}

impl<M> LambdayTerm<M, usize> {
    fn normalize_head(self) -> Self {
        self
        // todo!()
    }
}