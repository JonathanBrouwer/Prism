use std::collections::HashMap;
use itertools::Itertools;
use crate::jonla::jerror::{JError, JErrorEntry, Span};
use crate::lambday::lambday::LambdayTerm;
use crate::lambday::lambday::LambdayTerm::*;

#[derive(Debug, Clone)]
pub struct TypeCheckMeta {
    span: Span,
    typ: Option<LambdayTerm<Span, usize>>,
}

impl LambdayTerm<Span, usize> {
    pub fn type_check(self) -> Result<LambdayTerm<TypeCheckMeta, usize>, JError> {
        let mut errors = Vec::new();
        let term = self.type_check_inner(&mut HashMap::new(), &mut errors);
        if errors.len() == 0 {
            Ok(term)
        } else {
            Err(JError { errors })
        }
    }

    fn type_check_inner(self, names: &mut HashMap<usize, Option<LambdayTerm<Span, usize>>>, errors: &mut Vec<JErrorEntry>) -> LambdayTerm<TypeCheckMeta, usize> {
        match self {
            Var(span, name) => {
                Var(TypeCheckMeta { span, typ: names.get(&name).unwrap().clone() }, name)
            }
            TypeType(span) => {
                TypeType(TypeCheckMeta { span, typ: Some(self) })
            }
            FunType(span, arg_type, body_type) => {
                let arg_type: LambdayTerm<TypeCheckMeta, usize> = arg_type.type_check_inner(names, errors);
                let body_type: LambdayTerm<TypeCheckMeta, usize> = body_type.type_check_inner(names, errors);
                if let Some(t) = &arg_type.meta().typ { t.check_is_type_type(names, errors); }
                if let Some(t) = &body_type.meta().typ { t.check_is_type_type(names, errors); }

                FunType(TypeCheckMeta { span, typ: Some(TypeType(span)) }, box arg_type, box body_type)
            }
            FunConstr(span, sym, arg_type, body) => {
                let arg_type_checked: LambdayTerm<TypeCheckMeta, usize> = arg_type.clone().type_check_inner(names, errors);

                //Calc body type
                if arg_type_checked.meta().typ.is_some() && arg_type_checked.meta().typ.as_ref().unwrap().check_is_type_type(names, errors) {
                    names.insert(sym, Some((*arg_type).clone()));
                } else {
                    names.insert(sym, None);
                }
                let body: LambdayTerm<TypeCheckMeta, usize> = body.type_check_inner(names, errors);
                names.remove(&sym);

                //If body has a valid type return function type to body
                let typ = if body.meta().typ.is_some() {
                    Some(FunType(span, box (*arg_type).clone(), box body.meta().typ.as_ref().unwrap().clone()))
                } else {
                    None
                };
                FunConstr(TypeCheckMeta { span, typ }, sym, box arg_type_checked, box body)
            }
            FunDestr(span, fun, arg) => {
                let fun: LambdayTerm<TypeCheckMeta, usize> = fun.type_check_inner(names, errors);
                let arg: LambdayTerm<TypeCheckMeta, usize> = arg.type_check_inner(names, errors);

                if fun.meta().typ.is_none() {
                    return FunDestr(TypeCheckMeta { span, typ: None }, box fun, box arg);
                }
                return if let FunType(_, fun_arg_type, fun_body_type) = fun.meta().typ.clone().unwrap().normalize_head() {
                    arg.meta().typ.as_ref().unwrap().check_is_type_eq_to(fun_arg_type.as_ref(), names, errors);

                    let typ: LambdayTerm<Span, usize> = *fun_body_type;
                    FunDestr(TypeCheckMeta { span, typ: Some(typ) }, box fun, box arg)
                } else {
                    errors.push(JErrorEntry::TypeExpectFunc(fun.meta().span));
                    FunDestr(TypeCheckMeta { span, typ: None }, box fun, box arg)
                };
            }
            ProdType(span, subtypes) => {
                let subtypes = subtypes.into_iter().map(|subtype| {
                    let subtype: LambdayTerm<TypeCheckMeta, usize> = subtype.type_check_inner(names, errors);
                    if let Some(t) = &subtype.meta().typ { t.check_is_type_type(names, errors); }
                    subtype
                }).collect();

                ProdType(TypeCheckMeta { span, typ: Some(TypeType(span)) }, subtypes)
            }
            ProdConstr(span, typ, values) => {
                let typ: Box<LambdayTerm<Span, usize>> = typ;
                let typ_checked: LambdayTerm<TypeCheckMeta, usize> = typ.clone().type_check_inner(names, errors);
                let values: Vec<LambdayTerm<TypeCheckMeta, usize>> = values.into_iter().map(|v| v.type_check_inner(names, errors)).collect();

                match typ.clone().normalize_head() {
                    ProdType(_, subtypes) => {
                        if values.len() != subtypes.len() {
                            //TODO specifically highlight too little/many arguments
                            errors.push(JErrorEntry::TypeWrongArgumentCount(span, subtypes.len(), values.len()))
                        } else {
                            for (val, sub) in values.iter().zip_eq(subtypes.iter()) {
                                if let Some(t) = &val.meta().typ { t.check_is_type_eq_to(&sub, names, errors); }
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
                let val: LambdayTerm<TypeCheckMeta, usize> = val.type_check_inner(names, errors);
                let typ: Option<LambdayTerm<Span, usize>> = match &val.meta().typ {
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
                    let subtype: LambdayTerm<TypeCheckMeta, usize> = subtype.type_check_inner(names, errors);
                    if let Some(t) = &subtype.meta().typ { t.check_is_type_type(names, errors); }
                    subtype
                }).collect();

                SumType(TypeCheckMeta { span, typ: Some(TypeType(span)) }, subtypes)
            }
            SumConstr(span, typ, num, val) => {
                let typ: Box<LambdayTerm<Span, usize>> = typ;
                let typ_checked: LambdayTerm<TypeCheckMeta, usize> = typ.clone().type_check_inner(names, errors);
                let val: LambdayTerm<TypeCheckMeta, usize> = val.type_check_inner(names, errors);

                match typ.clone().normalize_head() {
                    SumType(_, subs) => {
                        if num >= subs.len() {
                            errors.push(JErrorEntry::TypeInvalidNumber(span))
                        }
                        if let Some(t) = &val.meta().typ { t.check_is_type_eq_to(&subs[num], names, errors); }
                        SumConstr(TypeCheckMeta { span, typ: Some((*typ).clone()) }, box typ_checked, num, box val)
                    }
                    _ => {
                        errors.push(JErrorEntry::TypeExpectSum(*typ.meta()));
                        SumConstr(TypeCheckMeta { span, typ: None }, box typ_checked, num, box val)
                    }
                }
            }
            SumDestr(span, val, into_type, opts) => {
                let into_type_checked: LambdayTerm<TypeCheckMeta, usize> = into_type.clone().type_check_inner(names, errors);
                let val: LambdayTerm<TypeCheckMeta, usize> = val.type_check_inner(names, errors);
                let opts: Vec<LambdayTerm<TypeCheckMeta, usize>> = opts.into_iter().map(|v| v.type_check_inner(names, errors)).collect();

                if into_type_checked.meta().typ.is_none() || !into_type_checked.meta().typ.as_ref().unwrap().check_is_type_type(names, errors) {
                    return SumDestr(TypeCheckMeta { span, typ: None }, box val, box into_type_checked, opts);
                }

                match &val.meta().typ {
                    Some(SumType(_, subtypes)) => {
                        if subtypes.len() != opts.len() {
                            errors.push(JErrorEntry::TypeWrongArgumentCount(span, subtypes.len(), opts.len()))
                        } else {
                            for (val, subtype) in opts.iter().zip_eq(subtypes.into_iter()) {
                                let exp: LambdayTerm<Span, usize> = FunType(*subtype.meta(), box (*subtype).clone(), into_type.clone());
                                if let Some(t) = &val.meta().typ { t.check_is_type_eq_to(&exp, names, errors); }
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

    fn check_is_type_eq_to(&self, other: &Self, _names: &mut HashMap<usize, Option<LambdayTerm<Span, usize>>>, errors: &mut Vec<JErrorEntry>) -> bool {
        true
    }

    fn check_is_type_type(&self, _names: &mut HashMap<usize, Option<LambdayTerm<Span, usize>>>, errors: &mut Vec<JErrorEntry>) -> bool {
        true
    }
}

impl<M> LambdayTerm<M, usize> {
    fn normalize_head(self) -> Self {
        self
        // todo!()
    }
}