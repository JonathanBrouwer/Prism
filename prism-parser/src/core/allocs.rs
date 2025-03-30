use bumpalo_herd::Herd;

#[derive(Default)]
pub struct OwnedAllocs {
    herd: Herd,
}

impl OwnedAllocs {
    fn borrow(&self) -> &Herd {
        &self.herd
    }
}

#[derive(Copy, Clone)]
pub struct Allocs<'arn> {
    bump: &'arn Herd,
}

impl<'arn> Allocs<'arn> {
    pub fn new(bump: &'arn OwnedAllocs) -> Self {
        Self {
            bump: bump.borrow(),
        }
    }

    pub fn new_leaking() -> Self {
        Self::new(Box::leak(Box::new(OwnedAllocs::default())))
    }

    pub fn alloc<T: Copy>(&self, t: T) -> &'arn T {
        self.bump.get().alloc(t)
    }

    pub fn alloc_str(&self, s: &str) -> &'arn str {
        self.bump.get().alloc_str(s)
    }

    pub fn alloc_extend<T: Copy, I: IntoIterator<Item = T, IntoIter: ExactSizeIterator>>(
        &self,
        iter: I,
    ) -> &'arn [T] {
        self.bump.get().alloc_slice_fill_iter(iter)
    }

    pub fn alloc_extend_len<T: Copy, I: IntoIterator<Item = T>>(
        &self,
        len: usize,
        iter: I,
    ) -> &'arn [T] {
        let mut iter = iter.into_iter();
        let slice = self.bump.get().alloc_slice_fill_with(len, |_| {
            iter.next().expect("Iterator supplied too few elements")
        });
        assert!(iter.next().is_none());
        slice
    }
}
