use std::sync::Arc;

pub fn alloc_extend<T, I: IntoIterator<Item = T>>(iter: I) -> Arc<[T]> {
    Arc::from_iter(iter)
}
