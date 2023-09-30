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
    /// A linked list of all free allocations
    /// Invariant: The pointer is a unique and valid pointer that has enough space to store a `RcInner`.
    free: Cell<Option<NonNull<RcFree>>>,
}

/// This struct contains the information required for reference counting
/// Invariant: The `count` stores the amount of `Rc` structs that have a pointer to this struct.
struct RcInner<'a, T> {
    data: T,
    count: Cell<usize>,
    arena: &'a RcArena<'a, T>,
}

/// This struct contains a pointer to the next free allocation, if any
/// Invariant: The pointer is a unique and valid pointer that has enough space to store a `RcInner`.
struct RcFree(Option<NonNull<RcFree>>);

impl<'a, T> RcArena<'a, T> {
    /// Creates a new Rc arena using a given bump allocator
    pub fn new(alloc: &'a Bump) -> Self {
        Self {
            alloc,
            phantom: PhantomData,
            free: Cell::new(None),
        }
    }

    /// Allocates a new `T` in the arena, returning the `Rc`
    pub fn alloc(&'a self, data: T) -> Rc<'a, T> {
        // Create a new Rc, the reference count starts at 1
        let inner = RcInner {
            data,
            count: Cell::new(1),
            arena: self,
        };

        match self.free.get() {
            // There is nothing free, so create a new allocation in the bump allocator
            None => Rc(NonNull::from(self.alloc.alloc(inner))),
            // There is a slot `p` available
            Some(p) => unsafe {
                // Set the `free` pointer to the next free allocation
                // Safety: p is a valid pointer.
                self.free.set(p.as_ref().0);
                // Safety: p was allocated as a `RcInner` (the invariant for `RcFree`) so it is safe to transmute it back into one.
                // It is non-null since it is a valid allocation.
                let mut p: NonNull<RcInner<'a, T>> = mem::transmute(p);
                // Safety: p is a valid pointer because of the invariant of `Rc`.
                *p.as_mut() = inner;
                // Safety: The reference count is 1, which is the exact amount of `Rc`s which have a reference to `p` (only this one).
                Rc(p)
            },
        }
    }
}

/// This struct contains a pointer to the next free allocation, if any
/// Invariant: The pointer is a valid pointer that has enough space to store a `RcInner`.
pub struct Rc<'a, T>(NonNull<RcInner<'a, T>>);

impl<'a, T> Clone for Rc<'a, T> {
    fn clone(&self) -> Self {
        // Safety: p is a valid pointer, so we can safely dereference it.
        // We increment the reference count by one, then we can safely return a new `Rc` instance.
        let r = unsafe { self.0.as_ref() };
        r.count.set(r.count.get() + 1);
        Self(self.0)
    }
}

impl<'a, T> Drop for Rc<'a, T> {
    fn drop(&mut self) {
        // Safety: The `Rc` contains a valid pointer, so we can dereference it.
        unsafe {
            // We are dropping this Rc, so we decrement the count
            let count = &(*self.0.as_ptr()).count;
            count.set(count.get() - 1);
            // If we are the last Rc, we can add this to the free pool.
            if count.get() == 0 {
                // Drop the data. Taking a mutable reference is safe because no other Rc has a reference to this data anymore.
                let data = &mut (*self.0.as_ptr()).data;
                ptr::drop_in_place(data);

                // Add ourself to the free list. We transmute this allocation to a linked list entry.
                // Because it was a `RcInner` first, this is safe.
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
        // Safety, the pointer is valid since we have a reference to a `Rc`.
        unsafe { &(*self.0.as_ptr()).data }
    }
}

// pub mod global {
//     pub type RcArena<T> = super::RcArena<'static, T>;
//     pub type Rc<T> = super::Rc<'static, T>;
// }

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
