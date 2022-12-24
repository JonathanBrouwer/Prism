use bumpalo::Bump;
use std::cell::Cell;
use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr::NonNull;
use std::{mem, ptr};

/// A RcArena is an arena that gives out `Rc` instances that are allocated in a `Bump` allocator.
/// It re-uses allocations that have been freed, and drops the content when done.
pub struct RcArena<'a, T> {
    alloc: &'a Bump,
    phantom: PhantomData<T>,
    free: Cell<Option<NonNull<RcFree>>>,
}

impl<'a, T> RcArena<'a, T> {
    /// Creates a new Rc arena using a given bump allocator
    pub fn new(alloc: &'a Bump) -> Self {
        Self {
            alloc,
            phantom: PhantomData::default(),
            free: Cell::new(None),
        }
    }

    /// Allocates a new `T` in the arena, returning the `Rc`
    pub fn alloc(&'a self, data: T) -> Rc<'a, T> {
        // Create a new Rc, the reference count starts at 1
        let inner = RcInner {
            data,
            count: Cell::new(1),
            arena: &self,
        };

        match self.free.get() {
            // There is nothing free, so create a new allocation in the bump allocator
            None => Rc(NonNull::from(self.alloc.alloc(inner))),
            // There is a slot `p` available
            Some(p) => unsafe {

                self.free.set(p.as_ref().0);
                let mut p: NonNull<RcInner<'a, T>> = mem::transmute(p);
                *p.as_mut() = inner;
                Rc(p)
            },
        }
    }
}

pub struct Rc<'a, T>(NonNull<RcInner<'a, T>>);

impl<'a, T> Clone for Rc<'a, T> {
    fn clone(&self) -> Self {
        let r = unsafe { self.0.as_ref() };
        r.count.set(r.count.get() + 1);
        Self(self.0)
    }
}

impl<'a, T> Drop for Rc<'a, T> {
    fn drop(&mut self) {
        unsafe {
            let count = &(*self.0.as_ptr()).count;
            count.set(count.get() - 1);
            if count.get() == 0 {
                let data = &mut (*self.0.as_ptr()).data;
                ptr::drop_in_place(data);

                let arena = (*self.0.as_ptr()).arena;

                let free: *mut RcFree = mem::transmute(self.0);
                (*free).0 = arena.free.get();
                arena.free.set(Some(NonNull::new_unchecked(free)));
            }
        }
    }
}

impl<T> Deref for Rc<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &(*self.0.as_ptr()).data }
    }
}

pub struct RcInner<'a, T> {
    data: T,
    count: Cell<usize>,
    arena: &'a RcArena<'a, T>,
}

struct RcFree(Option<NonNull<RcFree>>);

pub mod global {
    pub type RcArena<T> = super::RcArena<'static, T>;
    pub type Rc<T> = super::Rc<'static, T>;
}

#[cfg(test)]
mod tests {
    use crate::arena::RcArena;
    use bumpalo::Bump;

    #[test]
    fn simple() {
        let bump = Bump::new();
        let arena = RcArena::new(&bump);

        let w = arena.alloc(14);

        let v = arena.alloc(15);
        assert_eq!(*v, 15);
        drop(v);

        let v = arena.alloc(16);
        assert_eq!(*v, 16);
        drop(v);

        assert_eq!(*w, 14);
    }
}
