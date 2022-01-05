use std::collections::HashMap;
use itertools::Itertools;
use crate::jonla::jerror::{JError, JErrorEntry, Span};
use crate::lambday::lambday::LambdayTerm;
use crate::lambday::lambday::LambdayTerm::*;

#[derive(Debug, Clone)]
pub struct TypeCheckMeta {
    span: Span,
    typ: LambdayTerm<Span, usize>
}

impl LambdayTerm<Span, usize> {

    pub fn type_check(self) -> Result<LambdayTerm<TypeCheckMeta, usize>, JError> {
        let mut errors = Vec::new();
        let term = self.type_check_inner(&mut HashMap::new(), &mut errors);
        if errors.len() == 0 {
            Ok(term)
        } else {
            Err(JError{errors})
        }
    }

    fn type_check_inner(self, names: &mut HashMap<usize, LambdayTerm<Span, usize>>, errors: &mut Vec<JErrorEntry>) -> LambdayTerm<TypeCheckMeta, usize> {
        match self {
            Var(span, name) => {
                let typ = names.get(&name).unwrap().clone();
                Var(TypeCheckMeta{span, typ}, name)
            }
            TypeType(span) => {
                let typ = self; //TODO inconsistent
                TypeType(TypeCheckMeta{span, typ})
            }
            FunType(span, arg_type, body_type) => {
                let arg_type: LambdayTerm<TypeCheckMeta, usize> = arg_type.type_check_inner(names, errors);
                let body_type: LambdayTerm<TypeCheckMeta, usize> = body_type.type_check_inner(names, errors);
                arg_type.check_is_type(names, errors);
                body_type.check_is_type(names, errors);

                let typ = TypeType(span);
                FunType(TypeCheckMeta{span, typ}, box arg_type, box body_type)
            }
            FunConstr(span, sym, arg_type, body) => {
                let arg_type: LambdayTerm<TypeCheckMeta, usize> = arg_type.type_check_inner(names, errors);
                arg_type.check_is_type(names, errors);

                //Calc body type
                names.insert(sym, arg_type.meta().typ.clone());
                let body = body.type_check_inner(names, errors);
                names.remove(&sym);

                let typ = FunType(span,box arg_type.meta().typ.clone(), box body.meta().typ.clone());
                FunConstr(TypeCheckMeta{span,typ}, sym, box arg_type, box body)
            },
            FunDestr(span, fun, arg) => {
                let fun: LambdayTerm<TypeCheckMeta, usize> = fun.type_check_inner(names, errors);
                let arg: LambdayTerm<TypeCheckMeta, usize> = arg.type_check_inner(names, errors);

                return if let FunType(_, fun_arg_type, fun_body_type) = fun.meta().typ.clone().normalize_head() {
                    arg.meta().typ.check_is_type_eq_to(fun_arg_type.as_ref(), names, errors);

                    let typ: LambdayTerm<Span, usize> = *fun_body_type;
                    FunDestr(TypeCheckMeta{span,typ}, box fun, box arg)
                } else {
                    errors.push(JErrorEntry::TypeExpectFunc(fun.meta().span));
                    todo!()
                }
            },
            ProdType(span, subtypes) => {
                let subtypes = subtypes.into_iter().map(|subtype| {
                    let subtype = subtype.type_check_inner(names, errors);
                    subtype.check_is_type(names, errors);
                    subtype
                }).collect();

                let typ = TypeType(span);
                ProdType(TypeCheckMeta{span, typ}, subtypes)
            },
            ProdConstr(span, typ, values) => {
                let typ: Box<LambdayTerm<Span, usize>> = typ;
                let typ_checked: LambdayTerm<TypeCheckMeta, usize> = typ.clone().type_check_inner(names, errors);
                let values: Vec<LambdayTerm<TypeCheckMeta, usize>> = values.into_iter().map(|v| v.type_check_inner(names, errors)).collect();

                match typ_checked.clone().normalize_head() {
                    ProdType(_, subtypes) => {
                        if values.len() != subtypes.len() {
                            //TODO specifically highlight too little/many arguments
                            errors.push(JErrorEntry::TypeWrongArgumentCount(span, subtypes.len(), values.len()))
                        } else {
                            for (val, sub) in values.iter().zip_eq(subtypes.iter()) {
                                val.meta().typ.check_is_type_eq_to(&sub.meta().typ, names, errors);
                            }
                        }

                        ProdConstr(TypeCheckMeta{span, typ: (*typ).clone()}, box typ_checked, values)
                    },
                    _ => {
                        errors.push(JErrorEntry::TypeExpectProd(*typ.meta()));
                        todo!()
                    }
                }
            }
            ProdDestr(span, val, num) => {
                let val: LambdayTerm<TypeCheckMeta, usize> = val.type_check_inner(names, errors);
                match &val.meta().typ {
                    ProdType(_, types) => {
                        if num >= types.len() {
                            errors.push(JErrorEntry::TypeInvalidNumber(span));
                            return todo!()
                        }

                        let typ = types[num].clone();
                        ProdDestr(TypeCheckMeta{span, typ}, box val, num)
                    }
                    _ => {
                        errors.push(JErrorEntry::TypeExpectProd(val.meta().span));
                        todo!()
                    }
                }
            }
            SumType(span, subtypes) => {
                let subtypes = subtypes.into_iter().map(|subtype| {
                    let subtype = subtype.type_check_inner(names, errors);
                    subtype.check_is_type(names, errors);
                    subtype
                }).collect();

                let typ = TypeType(span);
                SumType(TypeCheckMeta{span, typ}, subtypes)
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
                        val.meta().typ.check_is_type_eq_to(&subs[num], names, errors);
                        SumConstr(TypeCheckMeta{span, typ: (*typ).clone()}, box typ_checked, num, box val)
                    }
                    _ => {
                        errors.push(JErrorEntry::TypeExpectSum(*typ.meta()));
                        todo!()
                    }
                }
            }
            SumDestr(span, val, into_type, opts) => {
                let into_type_checked: LambdayTerm<TypeCheckMeta, usize> = into_type.clone().type_check_inner(names, errors);
                into_type_checked.check_is_type(names, errors);

                let val: LambdayTerm<TypeCheckMeta, usize> = val.type_check_inner(names, errors);
                let opts: Vec<LambdayTerm<TypeCheckMeta, usize>> = opts.into_iter().map(|v| v.type_check_inner(names, errors)).collect();

                match &val.meta().typ {
                    SumType(_, subtypes) => {
                        if subtypes.len() != opts.len() {
                            errors.push(JErrorEntry::TypeWrongArgumentCount(span, subtypes.len(), opts.len()))
                        }else {
                            for (val, subtype) in opts.iter().zip_eq(subtypes.into_iter()) {
                                let exp: LambdayTerm<Span, usize> = FunType(*subtype.meta(), box (*subtype).clone(), into_type.clone());
                                val.meta().typ.check_is_type_eq_to(&exp, names, errors);
                            }
                        }

                        SumDestr(TypeCheckMeta{span, typ: (*into_type).clone()}, box val, box into_type_checked, opts)
                    }
                    _ => {
                        errors.push(JErrorEntry::TypeExpectSum(val.meta().span));
                        todo!()
                    }
                }
            },
        }
    }

    fn check_is_type_eq_to(&self, other: &Self, _names: &mut HashMap<usize, LambdayTerm<Span, usize>>, errors: &mut Vec<JErrorEntry>) {
        // todo!()
    }
}

impl LambdayTerm<TypeCheckMeta, usize> {
    fn check_is_type(&self, _names: &mut HashMap<usize, LambdayTerm<Span, usize>>, errors: &mut Vec<JErrorEntry>) {
        if let TypeType(_) = self.meta().typ {

        } else {
            errors.push(JErrorEntry::TypeExpectType(self.meta().span))
        }
    }
}

impl<M> LambdayTerm<M, usize> {
    fn normalize_head(self) -> Self {
        self
        // todo!()
    }
}