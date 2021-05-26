pub fn slice_reference<T : Sized>(slice: &[T], item: &T) -> Option<usize> {
    // This is technically incorrect usage, so this function should be unsafe, but assuming this is running on a reasonable architecture, this can't crash.
    unsafe {
        let start_addr : *const T = slice.as_ptr();
        let item_addr : *const T = item;
        let diff = item_addr.offset_from(start_addr);
        if diff >= 0 && diff < slice.len() as isize {
            Some(diff as usize)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::util::slice_index::slice_reference;
    use std::borrow::Borrow;

    #[test]
    fn test_simple() {
        let slice = &[0usize,1,2,3,4,5];
        for i in 0..6 {
            assert_eq!(slice_reference(slice, &slice[i]), Some(i));
        }
        let not_in_slice = 0;
        assert_eq!(slice_reference(slice, &not_in_slice), None);
        let bx = Box::new(0usize);
        assert_eq!(slice_reference(slice, bx.borrow()), None);
    }
}