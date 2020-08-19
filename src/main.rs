#![feature(min_const_generics)]
// #![feature(const_fn)]

use core::{mem::MaybeUninit, ptr, slice};
struct Vec<T, const N: usize> {
    buffer: MaybeUninit<[T; N]>,
    len: usize,
}

impl<T, const N: usize> Vec<T, N>
where
    T: Sized,
{
    pub(crate) const fn new() -> Self {
        Vec {
            buffer: MaybeUninit::uninit(),
            len: 0,
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.len
    }

    pub(crate) fn as_slice(&self) -> &[T] {
        // NOTE(unsafe) avoid bound checks in the slicing operation
        // &buffer[..self.len]
        unsafe { slice::from_raw_parts(self.buffer.as_ptr() as *const T, self.len) }
    }

    pub(crate) fn as_mut_slice(&mut self) -> &mut [T] {
        // NOTE(unsafe) avoid bound checks in the slicing operation
        // &mut buffer[..len]
        unsafe { slice::from_raw_parts_mut(self.buffer.as_mut_ptr() as *mut T, self.len) }
    }

    pub(crate) fn capacity(&self) -> usize {
        N
    }

    pub(crate) fn clear(&mut self) {
        self.truncate(0);
    }

    pub(crate) fn clone(&self) -> Self
    where
        T: Clone,
    {
        let mut new = Self::new();
        new.extend_from_slice(self.as_slice()).unwrap();
        new
    }

    pub(crate) fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        for elem in iter {
            self.push(elem).ok().unwrap()
        }
    }

    pub(crate) fn extend_from_slice(&mut self, other: &[T]) -> Result<(), ()>
    where
        T: Clone,
    {
        if self.len + other.len() > self.capacity() {
            // won't fit in the `Vec`; don't modify anything and return an error
            Err(())
        } else {
            for elem in other {
                unsafe {
                    self.push_unchecked(elem.clone());
                }
            }
            Ok(())
        }
    }

    pub(crate) fn is_full(&self) -> bool {
        self.len == self.capacity()
    }

    pub(crate) unsafe fn pop_unchecked(&mut self) -> T {
        debug_assert!(!self.as_slice().is_empty());

        self.len -= 1;
        (self.buffer.as_ptr() as *const T).add(self.len).read()
    }

    pub(crate) fn push(&mut self, item: T) -> Result<(), T> {
        if self.len < self.capacity() {
            unsafe { self.push_unchecked(item) }
            Ok(())
        } else {
            Err(item)
        }
    }

    pub(crate) unsafe fn push_unchecked(&mut self, item: T) {
        // NOTE(ptr::write) the memory slot that we are about to write to is uninitialized. We
        // use `ptr::write` to avoid running `T`'s destructor on the uninitialized memory
        (self.buffer.as_mut_ptr() as *mut T)
            .add(self.len)
            .write(item);

        self.len += 1;
    }

    unsafe fn swap_remove_unchecked(&mut self, index: usize) -> T {
        let length = self.len;
        debug_assert!(index < length);
        ptr::swap(
            self.as_mut_slice().get_unchecked_mut(index),
            self.as_mut_slice().get_unchecked_mut(length - 1),
        );
        self.pop_unchecked()
    }

    pub(crate) fn swap_remove(&mut self, index: usize) -> T {
        assert!(index < self.len);
        unsafe { self.swap_remove_unchecked(index) }
    }

    pub(crate) fn truncate(&mut self, len: usize) {
        unsafe {
            // drop any extra elements
            while len < self.len {
                // decrement len before the drop_in_place(), so a panic on Drop
                // doesn't re-drop the just-failed value.
                self.len -= 1;
                let len = self.len;
                ptr::drop_in_place(self.as_mut_slice().get_unchecked_mut(len));
            }
        }
    }
}

static mut V: Vec<u8, 8> = Vec::new();

fn main() {
    unsafe {
        assert!(V.len() == 0);
        V.push(1);
        assert!(V.len() == 1)
    }
}
