use crate::ast::abstract_ast::*;

pub struct Name {
    name: String,
    location: SrcSpan
}

pub struct NameType {
    name: Name,
    type_info: AstIndex
}

pub struct FnType {
    inputs: Vec<NameType>,
    output: NameType
}

pub enum BaseAst {
    DefineId {
        location: SrcSpan,
        name: Name,
        value: AstIndex,
    },
    RetrieveId {
        location: SrcSpan,
        name: Name,
    },
    FunctionType {
        location: SrcSpan,
        fn_type: FnType,
    },
    Function {
        location: SrcSpan,
        fn_type: FnType,
        body: AstIndex
    },
    Call {
        location: SrcSpan,
        function: AstIndex,
        args: AstIndex,
    },
    MultType {
        location: SrcSpan,
        values: Vec<NameType>
    },
    AddType {
        location: SrcSpan,
        values: Vec<NameType>
    },
    Case {
        location: SrcSpan,
        value: AstIndex,
        name: Name,
        cases: Vec<AstIndex>
    },
    Sequence {
        location: SrcSpan,
        expressions: Vec<AstIndex>,
    }
}

impl Ast for BaseAst {
    fn location(&self) -> SrcSpan {
        *match self {
            BaseAst::DefineId { location, .. } => location,
            BaseAst::RetrieveId { location, .. } => location,
            BaseAst::Function { location, .. } => location,
            BaseAst::Call { location, .. } => location,
            BaseAst::MultType { location, .. } => location,
            BaseAst::AddType { location, .. } => location,
            BaseAst::Case { location, .. } => location,
            BaseAst::Sequence { location, .. } => location
        }
    }
}