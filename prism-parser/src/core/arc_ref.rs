use std::ops::Deref;
use std::slice::SliceIndex;
use std::sync::Arc;

#[derive(Clone)]
pub struct ArcSlice<T> {
    owner: Arc<[T]>,
    reference: *const [T],
}

impl<T> ArcSlice<T> {
    pub fn new(owner: Arc<[T]>) -> ArcSlice<T> {
        Self {
            reference: owner.as_ref(),
            owner,
        }
    }

    pub fn to_borrowed(&self) -> BorrowedArcSlice<'_, T> {
        BorrowedArcSlice {
            owner: &self.owner,
            reference: self.reference,
        }
    }
}

unsafe impl<T> Send for ArcSlice<T> where Arc<[T]>: Send {}
unsafe impl<T> Sync for ArcSlice<T> where Arc<[T]>: Sync {}

pub struct BorrowedArcSlice<'a, T> {
    owner: &'a Arc<[T]>,
    reference: *const [T],
}

impl<'a, T> BorrowedArcSlice<'a, T> {
    pub fn new(owner: &'a Arc<[T]>) -> Self {
        Self {
            reference: owner.as_ref(),
            owner,
        }
    }

    pub fn to_cloned(&self) -> ArcSlice<T> {
        ArcSlice {
            owner: self.owner.clone(),
            reference: self.reference,
        }
    }

    pub fn slice(&self, range: impl SliceIndex<[T], Output = [T]>) -> Self {
        Self {
            owner: self.owner,
            reference: &unsafe { &*self.reference }[range],
        }
    }
}

impl<'a, T> Deref for BorrowedArcSlice<'a, T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.reference }
    }
}

impl<'a, T> Clone for BorrowedArcSlice<'a, T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<'a, T> Copy for BorrowedArcSlice<'a, T> {}

unsafe impl<'a, T> Send for BorrowedArcSlice<'a, T> where &'a Arc<[T]>: Send {}
unsafe impl<'a, T> Sync for BorrowedArcSlice<'a, T> where &'a Arc<[T]>: Sync {}
