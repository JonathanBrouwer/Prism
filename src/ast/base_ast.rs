use crate::ast::ast::*;

#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub struct SrcSpan {
    start: usize,
    end: usize,
}

#[derive(Clone)]
pub struct Name {
    pub name: String,
    pub location: SrcSpan
}

#[derive(Clone)]
pub struct Ast {
    pub location: SrcSpan,
    pub sub: AstSub
}

#[derive(Clone)]
pub enum AstSub {
    DefineId {
        name: Name,
        value: AstIndex,
    },
    RetrieveId {
        name: Name,
    },
    Function {
        inputs: Vec<Name>,
        body: AstIndex
    },
    Call {
        function: AstIndex,
        args: Vec<AstIndex>,
    },
    FunctionType {
        inputs: Vec<AstIndex>,
        output: AstIndex,
    },
    MultType {
        values: Vec<AstIndex>,
    },
    AddType {
        values: Vec<AstIndex>,
    },
    Case {
        value: AstIndex,
        name: Name,
        cases: Vec<AstIndex>
    },
    Sequence {
        expressions: Vec<AstIndex>,
    }
}

impl AstSub {
    pub fn children(&self) -> Vec<AstIndex> {
        match self {
            AstSub::DefineId { value, .. } => vec![*value],
            AstSub::RetrieveId { .. } => vec![],
            AstSub::Function { body, .. } => vec![*body],
            AstSub::Call { function, args } => {
                let mut res = vec![];
                res.push(*function);
                res.extend(args);
                res
            },
            AstSub::FunctionType { inputs, output} => {
                let mut res = vec![];
                res.extend(inputs.iter().to_owned());
                res.push(*output);
                res
            },
            AstSub::MultType { values } => values.clone(),
            AstSub::AddType { values } => values.clone(),
            AstSub::Case { value, cases, .. } => {
                let mut res = vec![];
                res.extend(cases.iter().to_owned());
                res.push(*value);
                res
            },
            AstSub::Sequence { expressions } => expressions.clone(),
        }
    }
}

pub type BaseAstCollection = AstCollection<Ast>;