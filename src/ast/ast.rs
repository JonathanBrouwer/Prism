use std::ops::{Index, IndexMut};

#[derive(Copy, Clone)]
pub struct AstIndex {
    index: usize
}

pub struct AstCollection<E : Clone> {
    vec: Vec<E>,
    pub start: AstIndex
}

impl<E : Clone> Index<AstIndex> for AstCollection<E> {
    type Output = E;

    fn index(&self, index: AstIndex) -> &Self::Output {
        &self.vec[index.index]
    }
}

impl<E : Clone> IndexMut<AstIndex> for AstCollection<E> {
    fn index_mut(&mut self, index: AstIndex) -> &mut Self::Output {
        &mut self.vec[index.index]
    }
}

impl<E : Clone> AstCollection<E> {
    pub fn create_empty_derivative<F : Clone + Default>(&self) -> AstCollection<F> {
        AstCollection::<F> { vec: vec![F::default(); self.vec.len()], start: self.start }
    }
}
