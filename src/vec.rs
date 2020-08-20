// #![feature(const_fn)]

use core::{mem::MaybeUninit, ptr, slice};
pub struct Vec<T, const N: usize> {
    buffer: MaybeUninit<[T; N]>,
    len: usize,
}

impl<T, const N: usize> Vec<T, N>
where
    T: Sized,
{
    pub const fn new() -> Self {
        Vec {
            buffer: MaybeUninit::uninit(),
            len: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn as_slice(&self) -> &[T] {
        // NOTE(unsafe) avoid bound checks in the slicing operation
        // &buffer[..self.len]
        unsafe { slice::from_raw_parts(self.buffer.as_ptr() as *const T, self.len) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        // NOTE(unsafe) avoid bound checks in the slicing operation
        // &mut buffer[..len]
        unsafe { slice::from_raw_parts_mut(self.buffer.as_mut_ptr() as *mut T, self.len) }
    }

    pub fn capacity(&self) -> usize {
        N
    }

    pub fn clear(&mut self) {
        self.truncate(0);
    }

    pub fn clone(&self) -> Self
    where
        T: Clone,
    {
        let mut new = Self::new();
        new.extend_from_slice(self.as_slice()).unwrap();
        new
    }

    pub fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        for elem in iter {
            self.push(elem).ok().unwrap()
        }
    }

    pub fn extend_from_slice(&mut self, other: &[T]) -> Result<(), ()>
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

    pub fn is_full(&self) -> bool {
        self.len == self.capacity()
    }

    pub unsafe fn pop_unchecked(&mut self) -> T {
        debug_assert!(!self.as_slice().is_empty());

        self.len -= 1;
        (self.buffer.as_ptr() as *const T).add(self.len).read()
    }

    pub fn push(&mut self, item: T) -> Result<(), T> {
        if self.len < self.capacity() {
            unsafe { self.push_unchecked(item) }
            Ok(())
        } else {
            Err(item)
        }
    }

    pub unsafe fn push_unchecked(&mut self, item: T) {
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

    pub fn swap_remove(&mut self, index: usize) -> T {
        assert!(index < self.len);
        unsafe { self.swap_remove_unchecked(index) }
    }

    pub fn truncate(&mut self, len: usize) {
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

// impl<T, N> Default for Vec<T, N>
// where
//     N: ArrayLength<T>,
// {
//     fn default() -> Self {
//         Self::new()
//     }
// }

// impl<T, N> fmt::Debug for Vec<T, N>
// where
//     T: fmt::Debug,
//     N: ArrayLength<T>,
// {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         <[T] as fmt::Debug>::fmt(self, f)
//     }
// }

// impl<N> fmt::Write for Vec<u8, N>
// where
//     N: ArrayLength<u8>,
// {
//     fn write_str(&mut self, s: &str) -> fmt::Result {
//         match self.extend_from_slice(s.as_bytes()) {
//             Ok(()) => Ok(()),
//             Err(_) => Err(fmt::Error),
//         }
//     }
// }

impl<T, const N: usize> Drop for Vec<T, N> {
    fn drop(&mut self) {
        unsafe {
            ptr::drop_in_place(self.as_mut_slice());
        }
    }
}

// impl<T, N> Extend<T> for Vec<T, N>
// where
//     N: ArrayLength<T>,
// {
//     fn extend<I>(&mut self, iter: I)
//     where
//         I: IntoIterator<Item = T>,
//     {
//         self.0.extend(iter)
//     }
// }

// impl<'a, T, N> Extend<&'a T> for Vec<T, N>
// where
//     T: 'a + Copy,
//     N: ArrayLength<T>,
// {
//     fn extend<I>(&mut self, iter: I)
//     where
//         I: IntoIterator<Item = &'a T>,
//     {
//         self.extend(iter.into_iter().cloned())
//     }
// }

// impl<T, N> hash::Hash for Vec<T, N>
// where
//     T: core::hash::Hash,
//     N: ArrayLength<T>,
// {
//     fn hash<H: hash::Hasher>(&self, state: &mut H) {
//         <[T] as hash::Hash>::hash(self, state)
//     }
// }

// impl<T, N> hash32::Hash for Vec<T, N>
// where
//     T: hash32::Hash,
//     N: ArrayLength<T>,
// {
//     fn hash<H: hash32::Hasher>(&self, state: &mut H) {
//         <[T] as hash32::Hash>::hash(self, state)
//     }
// }

// impl<'a, T, N> IntoIterator for &'a Vec<T, N>
// where
//     N: ArrayLength<T>,
// {
//     type Item = &'a T;
//     type IntoIter = slice::Iter<'a, T>;

//     fn into_iter(self) -> Self::IntoIter {
//         self.iter()
//     }
// }

// impl<'a, T, N> IntoIterator for &'a mut Vec<T, N>
// where
//     N: ArrayLength<T>,
// {
//     type Item = &'a mut T;
//     type IntoIter = slice::IterMut<'a, T>;

//     fn into_iter(self) -> Self::IntoIter {
//         self.iter_mut()
//     }
// }

// impl<T, N> FromIterator<T> for Vec<T, N>
// where
//     N: ArrayLength<T>,
// {
//     fn from_iter<I>(iter: I) -> Self
//     where
//         I: IntoIterator<Item = T>,
//     {
//         let mut vec = Vec::new();
//         for i in iter {
//             vec.push(i).ok().expect("Vec::from_iter overflow");
//         }
//         vec
//     }
// }

// /// An iterator that moves out of an [`Vec`][`Vec`].
// ///
// /// This struct is created by calling the `into_iter` method on [`Vec`][`Vec`].
// ///
// /// [`Vec`]: (https://doc.rust-lang.org/std/vec/struct.Vec.html)
// ///
// pub struct IntoIter<T, N>
// where
//     N: ArrayLength<T>,
// {
//     vec: Vec<T, N>,
//     next: usize,
// }

// impl<T, N> Iterator for IntoIter<T, N>
// where
//     N: ArrayLength<T>,
// {
//     type Item = T;
//     fn next(&mut self) -> Option<Self::Item> {
//         if self.next < self.vec.len() {
//             let item = unsafe {
//                 (self.vec.0.buffer.as_ptr() as *const T)
//                     .add(self.next)
//                     .read()
//             };
//             self.next += 1;
//             Some(item)
//         } else {
//             None
//         }
//     }
// }

// impl<T, N> Clone for IntoIter<T, N>
// where
//     T: Clone,
//     N: ArrayLength<T>,
// {
//     fn clone(&self) -> Self {
//         Self {
//             vec: self.vec.clone(),
//             next: self.next,
//         }
//     }
// }

// impl<T, N> Drop for IntoIter<T, N>
// where
//     N: ArrayLength<T>,
// {
//     fn drop(&mut self) {
//         unsafe {
//             // Drop all the elements that have not been moved out of vec
//             ptr::drop_in_place(&mut self.vec[self.next..]);
//             // Prevent dropping of other elements
//             self.vec.0.len = 0;
//         }
//     }
// }

// impl<T, N> IntoIterator for Vec<T, N>
// where
//     N: ArrayLength<T>,
// {
//     type Item = T;
//     type IntoIter = IntoIter<T, N>;

//     fn into_iter(self) -> Self::IntoIter {
//         IntoIter { vec: self, next: 0 }
//     }
// }

// impl<A, B, N1, N2> PartialEq<Vec<B, N2>> for Vec<A, N1>
// where
//     N1: ArrayLength<A>,
//     N2: ArrayLength<B>,
//     A: PartialEq<B>,
// {
//     fn eq(&self, other: &Vec<B, N2>) -> bool {
//         <[A]>::eq(self, &**other)
//     }
// }

// macro_rules! eq {
//     ($Lhs:ty, $Rhs:ty) => {
//         impl<'a, 'b, A, B, N> PartialEq<$Rhs> for $Lhs
//         where
//             A: PartialEq<B>,
//             N: ArrayLength<A>,
//         {
//             fn eq(&self, other: &$Rhs) -> bool {
//                 <[A]>::eq(self, &other[..])
//             }
//         }
//     };
// }

// eq!(Vec<A, N>, [B]);
// eq!(Vec<A, N>, &'a [B]);
// eq!(Vec<A, N>, &'a mut [B]);

// macro_rules! array {
//     ($($N:expr),+) => {
//         $(
//             eq!(Vec<A, N>, [B; $N]);
//             eq!(Vec<A, N>, &'a [B; $N]);
//         )+
//     }
// }

// array!(
//     0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
//     26, 27, 28, 29, 30, 31, 32
// );

// impl<T, N> Eq for Vec<T, N>
// where
//     N: ArrayLength<T>,
//     T: Eq,
// {
// }

// impl<T, N> ops::Deref for Vec<T, N>
// where
//     N: ArrayLength<T>,
// {
//     type Target = [T];

//     fn deref(&self) -> &[T] {
//         self.0.as_slice()
//     }
// }

// impl<T, N> ops::DerefMut for Vec<T, N>
// where
//     N: ArrayLength<T>,
// {
//     fn deref_mut(&mut self) -> &mut [T] {
//         self.0.as_mut_slice()
//     }
// }

// impl<T, N> AsRef<Vec<T, N>> for Vec<T, N>
// where
//     N: ArrayLength<T>,
// {
//     #[inline]
//     fn as_ref(&self) -> &Self {
//         self
//     }
// }

// impl<T, N> AsMut<Vec<T, N>> for Vec<T, N>
// where
//     N: ArrayLength<T>,
// {
//     #[inline]
//     fn as_mut(&mut self) -> &mut Self {
//         self
//     }
// }

// impl<T, N> AsRef<[T]> for Vec<T, N>
// where
//     N: ArrayLength<T>,
// {
//     #[inline]
//     fn as_ref(&self) -> &[T] {
//         self
//     }
// }

// impl<T, N> AsMut<[T]> for Vec<T, N>
// where
//     N: ArrayLength<T>,
// {
//     #[inline]
//     fn as_mut(&mut self) -> &mut [T] {
//         self
//     }
// }

#[cfg(test)]
mod tests {
    use crate::vec::Vec;
    //     use as_slice::AsSlice;
    //     use core::fmt::Write;

    #[test]
    fn static_new() {
        static mut _V: Vec<i32, 4> = Vec::new();
    }

    #[test]
    fn stack_new() {
        let mut v: Vec<i32, 4> = Vec::new();
        drop(v);
    }

    macro_rules! droppable {
        () => {
            struct Droppable;
            impl Droppable {
                fn new() -> Self {
                    unsafe {
                        COUNT += 1;
                    }
                    Droppable
                }
            }
            impl Drop for Droppable {
                fn drop(&mut self) {
                    unsafe {
                        COUNT -= 1;
                    }
                }
            }

            static mut COUNT: i32 = 0;
        };
    }

    // #[test]
    // fn drop() {
    //     droppable!();

    //     {
    //         let mut v: Vec<Droppable, 2> = Vec::new();
    //         // v.push(Droppable::new()).ok().unwrap();
    //         // v.push(Droppable::new()).ok().unwrap();
    //         // v.pop().unwrap();
    //     }

    //     assert_eq!(unsafe { COUNT }, 0);

    //     {
    //         let mut v: Vec<Droppable, 2> = Vec::new();
    //         v.push(Droppable::new()).ok().unwrap();
    //         v.push(Droppable::new()).ok().unwrap();
    //     }

    //     assert_eq!(unsafe { COUNT }, 0);
    // }

    //     #[test]
    //     fn eq() {
    //         let mut xs: Vec<i32, U4> = Vec::new();
    //         let mut ys: Vec<i32, U8> = Vec::new();

    //         assert_eq!(xs, ys);

    //         xs.push(1).unwrap();
    //         ys.push(1).unwrap();

    //         assert_eq!(xs, ys);
    //     }

    //     #[test]
    //     fn full() {
    //         let mut v: Vec<i32, U4> = Vec::new();

    //         v.push(0).unwrap();
    //         v.push(1).unwrap();
    //         v.push(2).unwrap();
    //         v.push(3).unwrap();

    //         assert!(v.push(4).is_err());
    //     }

    //     #[test]
    //     fn iter() {
    //         let mut v: Vec<i32, U4> = Vec::new();

    //         v.push(0).unwrap();
    //         v.push(1).unwrap();
    //         v.push(2).unwrap();
    //         v.push(3).unwrap();

    //         let mut items = v.iter();

    //         assert_eq!(items.next(), Some(&0));
    //         assert_eq!(items.next(), Some(&1));
    //         assert_eq!(items.next(), Some(&2));
    //         assert_eq!(items.next(), Some(&3));
    //         assert_eq!(items.next(), None);
    //     }

    //     #[test]
    //     fn iter_mut() {
    //         let mut v: Vec<i32, U4> = Vec::new();

    //         v.push(0).unwrap();
    //         v.push(1).unwrap();
    //         v.push(2).unwrap();
    //         v.push(3).unwrap();

    //         let mut items = v.iter_mut();

    //         assert_eq!(items.next(), Some(&mut 0));
    //         assert_eq!(items.next(), Some(&mut 1));
    //         assert_eq!(items.next(), Some(&mut 2));
    //         assert_eq!(items.next(), Some(&mut 3));
    //         assert_eq!(items.next(), None);
    //     }

    //     #[test]
    //     fn collect_from_iter() {
    //         let slice = &[1, 2, 3];
    //         let vec = slice.iter().cloned().collect::<Vec<_, U4>>();
    //         assert_eq!(vec, slice);
    //     }

    //     #[test]
    //     #[should_panic]
    //     fn collect_from_iter_overfull() {
    //         let slice = &[1, 2, 3];
    //         let _vec = slice.iter().cloned().collect::<Vec<_, U2>>();
    //     }

    //     #[test]
    //     fn iter_move() {
    //         let mut v: Vec<i32, U4> = Vec::new();
    //         v.push(0).unwrap();
    //         v.push(1).unwrap();
    //         v.push(2).unwrap();
    //         v.push(3).unwrap();

    //         let mut items = v.into_iter();

    //         assert_eq!(items.next(), Some(0));
    //         assert_eq!(items.next(), Some(1));
    //         assert_eq!(items.next(), Some(2));
    //         assert_eq!(items.next(), Some(3));
    //         assert_eq!(items.next(), None);
    //     }

    //     #[test]
    //     fn iter_move_drop() {
    //         droppable!();

    //         {
    //             let mut vec: Vec<Droppable, U2> = Vec::new();
    //             vec.push(Droppable::new()).ok().unwrap();
    //             vec.push(Droppable::new()).ok().unwrap();
    //             let mut items = vec.into_iter();
    //             // Move all
    //             let _ = items.next();
    //             let _ = items.next();
    //         }

    //         assert_eq!(unsafe { COUNT }, 0);

    //         {
    //             let mut vec: Vec<Droppable, U2> = Vec::new();
    //             vec.push(Droppable::new()).ok().unwrap();
    //             vec.push(Droppable::new()).ok().unwrap();
    //             let _items = vec.into_iter();
    //             // Move none
    //         }

    //         assert_eq!(unsafe { COUNT }, 0);

    //         {
    //             let mut vec: Vec<Droppable, U2> = Vec::new();
    //             vec.push(Droppable::new()).ok().unwrap();
    //             vec.push(Droppable::new()).ok().unwrap();
    //             let mut items = vec.into_iter();
    //             let _ = items.next(); // Move partly
    //         }

    //         assert_eq!(unsafe { COUNT }, 0);
    //     }

    //     #[test]
    //     fn push_and_pop() {
    //         let mut v: Vec<i32, U4> = Vec::new();
    //         assert_eq!(v.len(), 0);

    //         assert_eq!(v.pop(), None);
    //         assert_eq!(v.len(), 0);

    //         v.push(0).unwrap();
    //         assert_eq!(v.len(), 1);

    //         assert_eq!(v.pop(), Some(0));
    //         assert_eq!(v.len(), 0);

    //         assert_eq!(v.pop(), None);
    //         assert_eq!(v.len(), 0);
    //     }

    //     #[test]
    //     fn resize_size_limit() {
    //         let mut v: Vec<u8, U4> = Vec::new();

    //         v.resize(0, 0).unwrap();
    //         v.resize(4, 0).unwrap();
    //         v.resize(5, 0).err().expect("full");
    //     }

    //     #[test]
    //     fn resize_length_cases() {
    //         let mut v: Vec<u8, U4> = Vec::new();

    //         assert_eq!(v.len(), 0);

    //         // Grow by 1
    //         v.resize(1, 0).unwrap();
    //         assert_eq!(v.len(), 1);

    //         // Grow by 2
    //         v.resize(3, 0).unwrap();
    //         assert_eq!(v.len(), 3);

    //         // Resize to current size
    //         v.resize(3, 0).unwrap();
    //         assert_eq!(v.len(), 3);

    //         // Shrink by 1
    //         v.resize(2, 0).unwrap();
    //         assert_eq!(v.len(), 2);

    //         // Shrink by 2
    //         v.resize(0, 0).unwrap();
    //         assert_eq!(v.len(), 0);
    //     }

    //     #[test]
    //     fn resize_contents() {
    //         let mut v: Vec<u8, U4> = Vec::new();

    //         // New entries take supplied value when growing
    //         v.resize(1, 17).unwrap();
    //         assert_eq!(v[0], 17);

    //         // Old values aren't changed when growing
    //         v.resize(2, 18).unwrap();
    //         assert_eq!(v[0], 17);
    //         assert_eq!(v[1], 18);

    //         // Old values aren't changed when length unchanged
    //         v.resize(2, 0).unwrap();
    //         assert_eq!(v[0], 17);
    //         assert_eq!(v[1], 18);

    //         // Old values aren't changed when shrinking
    //         v.resize(1, 0).unwrap();
    //         assert_eq!(v[0], 17);
    //     }

    //     #[test]
    //     fn resize_default() {
    //         let mut v: Vec<u8, U4> = Vec::new();

    //         // resize_default is implemented using resize, so just check the
    //         // correct value is being written.
    //         v.resize_default(1).unwrap();
    //         assert_eq!(v[0], 0);
    //     }

    //     #[test]
    //     fn write() {
    //         let mut v: Vec<u8, U4> = Vec::new();
    //         write!(v, "{:x}", 1234).unwrap();
    //         assert_eq!(&v[..], b"4d2");
    //     }

    //     #[test]
    //     fn extend_from_slice() {
    //         let mut v: Vec<u8, U4> = Vec::new();
    //         assert_eq!(v.len(), 0);
    //         v.extend_from_slice(&[1, 2]).unwrap();
    //         assert_eq!(v.len(), 2);
    //         assert_eq!(v.as_slice(), &[1, 2]);
    //         v.extend_from_slice(&[3]).unwrap();
    //         assert_eq!(v.len(), 3);
    //         assert_eq!(v.as_slice(), &[1, 2, 3]);
    //         assert!(v.extend_from_slice(&[4, 5]).is_err());
    //         assert_eq!(v.len(), 3);
    //         assert_eq!(v.as_slice(), &[1, 2, 3]);
    //     }

    //     #[test]
    //     fn from_slice() {
    //         // Successful construction
    //         let v: Vec<u8, U4> = Vec::from_slice(&[1, 2, 3]).unwrap();
    //         assert_eq!(v.len(), 3);
    //         assert_eq!(v.as_slice(), &[1, 2, 3]);

    //         // Slice too large
    //         assert!(Vec::<u8, U2>::from_slice(&[1, 2, 3]).is_err());
    //     }

    //     #[test]
    //     fn starts_with() {
    //         let v: Vec<_, U8> = Vec::from_slice(b"ab").unwrap();
    //         assert!(v.starts_with(&[]));
    //         assert!(v.starts_with(b""));
    //         assert!(v.starts_with(b"a"));
    //         assert!(v.starts_with(b"ab"));
    //         assert!(!v.starts_with(b"abc"));
    //         assert!(!v.starts_with(b"ba"));
    //         assert!(!v.starts_with(b"b"));
    //     }

    //     #[test]
    //     fn ends_with() {
    //         let v: Vec<_, U8> = Vec::from_slice(b"ab").unwrap();
    //         assert!(v.ends_with(&[]));
    //         assert!(v.ends_with(b""));
    //         assert!(v.ends_with(b"b"));
    //         assert!(v.ends_with(b"ab"));
    //         assert!(!v.ends_with(b"abc"));
    //         assert!(!v.ends_with(b"ba"));
    //         assert!(!v.ends_with(b"a"));
    //     }
}
