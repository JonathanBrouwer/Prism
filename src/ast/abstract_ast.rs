pub struct AstCollection<T : Ast> {
    pub vec: Vec<T>,
    pub start: AstIndex
}

impl<T : Ast> AstCollection<T> {
    pub fn get(&self, index: AstIndex) -> &T {
        &self.vec[index.index]
    }

    pub fn map<S : Ast>(self, f : F) -> AstCollection<S>
        where F : Fn(T) -> S {

        todo!()
    }
}

pub struct AstIndex {
    index: usize
}

pub trait Ast {
    fn location(&self) -> SrcSpan;
}

#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub struct SrcSpan {
    start: usize,
    end: usize,
}