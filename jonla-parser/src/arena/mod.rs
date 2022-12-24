// use std::cell::Cell;
// use std::marker::PhantomData;
// use std::mem;
// use std::mem::ManuallyDrop;
// use bumpalo::Bump;
//
// pub struct RcArena<'a, T> {
//     alloc: &'a Bump,
//     free: Cell<Option<&'a mut FreeRcInner<'a, T>>>,
// }
//
// impl<'a, T> RcArena<'a, T> {
//     pub fn new(alloc: &'a Bump) -> Self {
//         Self {
//             alloc,
//             free: Cell::new(None),
//         }
//     }
//
//     pub fn rc(&'a self, data: T) -> Rc<'a, T> {
//         match self.free.take() {
//             Some(v) => unsafe {
//                 self.free.set(mem::transmute(v.next.as_deref_mut()));
//                 Rc(mem::transmute(v))
//             }
//             None => {
//                 Rc(self.alloc.alloc(AllocRcInner {
//                     data,
//                     count: Cell::new(1),
//                     arena: &self,
//                 }))
//             }
//         }
//     }
// }
//
// pub struct Rc<'a, T>(&'a AllocRcInner<'a, T>);
//
// impl<'a, T> Clone for Rc<'a, T> {
//     fn clone(&self) -> Self {
//         self.0.count.set(self.0.count.get() + 1);
//         Self(self.0)
//     }
// }
//
// impl<'a, T> Drop for Rc<'a, T> {
//     fn drop(&mut self) {
//         self.0.count.set(self.0.count.get() - 1);
//         if self.0.count.get() == 0 {
//             let arena = self.0.arena;
//             let freed: &mut FreeRcInner<'a, T> = unsafe { mem::transmute(self.0) };
//             // arena.free.set(Some())
//         }
//     }
// }
//
// impl<T> AsRef<T> for Rc<'_, T> {
//     fn as_ref(&self) -> &T {
//         &self.0.data
//     }
// }
//
// pub union RcInner<'a, T> {
//     alloc: AllocRcInner<'a, T>,
//     free: FreeRcInner<'a, T>,
// }
//
// pub struct AllocRcInner<'a, T> {
//     data: ManuallyDrop<T>,
//     count: Cell<usize>,
//     arena: &'a RcArena<'a, T>,
// }
//
// struct FreeRcInner<'a, T> {
//     next: Option<&'a mut FreeRcInner<'a, T>>,
//     phantom: PhantomData<T>,
// }