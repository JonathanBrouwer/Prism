use bumpalo::Bump;
use bumpalo_try::BumpaloExtend;

#[derive(Copy, Clone)]
pub struct Allocs<'arn> {
    bump: &'arn Bump,
}

impl<'arn> Allocs<'arn> {
    pub fn new(bump: &'arn Bump) -> Self {
        Self { bump }
    }

    pub fn new_leaking() -> Self {
        Self {
            bump: Box::leak(Box::new(Bump::new())),
        }
    }

    pub fn alloc<T: Copy>(&self, t: T) -> &'arn mut T {
        self.bump.alloc(t)
    }

    pub fn alloc_str(&self, s: &str) -> &'arn mut str {
        self.bump.alloc_str(s)
    }

    pub fn alloc_extend<T: Copy, I: IntoIterator<Item = T, IntoIter: ExactSizeIterator>>(
        &self,
        iter: I,
    ) -> &'arn mut [T] {
        self.bump.alloc_slice_fill_iter(iter)
    }

    pub fn alloc_extend_len<T: Copy, I: IntoIterator<Item = T>>(
        &self,
        len: usize,
        iter: I,
    ) -> &'arn mut [T] {
        let mut iter = iter.into_iter();
        let slice = self.bump.alloc_slice_fill_with(len, |_| {
            iter.next().expect("Iterator supplied too few elements")
        });
        assert!(iter.next().is_none());
        slice
    }

    pub fn try_alloc_extend_option<
        T: Copy,
        I: IntoIterator<Item = Option<T>, IntoIter: ExactSizeIterator>,
    >(
        &self,
        iter: I,
    ) -> Option<&'arn mut [T]> {
        self.bump.alloc_slice_fill_iter_option(iter)
    }

    pub fn try_alloc_extend_result<
        T: Copy,
        E,
        I: IntoIterator<Item = Result<T, E>, IntoIter: ExactSizeIterator>,
    >(
        &self,
        iter: I,
    ) -> Result<&'arn mut [T], E> {
        self.bump.alloc_slice_fill_iter_result(iter)
    }
}
