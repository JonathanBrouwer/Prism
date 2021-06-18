use crate::ast::abstract_ast::*;
use crate::ast::base_ast::*;
use std::collections::HashMap;

pub enum NamedAst {
    DefineId {
        location: SrcSpan,
        name: Name,
        value: AstIndex,
    },
    RetrieveId {
        location: SrcSpan,
        name: Name,
        refers_to: AstIndex
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
        values: Vec<AstIndex>
    },
    AddType {
        location: SrcSpan,
        values: Vec<AstIndex>
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

impl Ast for NamedAst {
    fn location(&self) -> SrcSpan {
        *match self {
            NamedAst::DefineId { location, .. } => location,
            NamedAst::RetrieveId { location, .. } => location,
            NamedAst::Function { location, .. } => location,
            NamedAst::Call { location, .. } => location,
            NamedAst::MultType { location, .. } => location,
            NamedAst::AddType { location, .. } => location,
            NamedAst::Case { location, .. } => location,
            NamedAst::Sequence { location, .. } => location
        }
    }
}

pub fn base_ast_to_named(input: AstCollection<BaseAst>) -> AstCollection<NamedAst> {
    struct Environment {
        name_map: HashMap<String, AstIndex>
    }


    todo!()
}