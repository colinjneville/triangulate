use std::{fmt, marker::PhantomData, num::NonZeroUsize};

#[derive(Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct Idx<T>(NonZeroUsize, PhantomData<T>);

impl<T> fmt::Debug for Idx<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

pub trait IdxDisplay {
    fn fmt(f: &mut std::fmt::Formatter<'_>, idx: usize) -> std::fmt::Result;
}

impl<T: IdxDisplay> fmt::Display for Idx<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        T::fmt(f, self.usize())
    }
}

impl<T> Idx<T> {
    pub fn new(index: usize) -> Self {
        let index = index + 1;
        Self(unsafe { NonZeroUsize::new_unchecked(index) }, Default::default())
    }

    pub fn advance(&mut self) -> Self {
        let curr = self.clone();
        *self = Self::new(self.usize() + 1);
        curr
    }

    pub fn advance_wrapped(&mut self, slice: &[T]) -> Self {
        let curr = self.clone();
        *self = self.next_wrapped(slice);
        curr
    }

    pub fn next_wrapped(&self, slice: &[T]) -> Self {
        if slice.len() == 0 {
            panic!("slice must not be empty");
        }
        Self::new((self.usize() + 1) % slice.len())
    }

    pub fn recede(&mut self) -> Self {
        let curr = self.clone();
        // Maybe just panic instead if already at 0?
        if self.usize() > 0 {
            *self = Self::new(self.usize() - 1);
        }
        curr
    }

    pub fn recede_wrapped(&mut self, slice: &[T]) -> Self {
        let curr = self.clone();
        *self = self.prev_wrapped(slice);
        curr
    }

    pub fn prev_wrapped(&self, slice: &[T]) -> Self {
        if slice.len() == 0 {
            panic!("slice must not be empty");
        }

        let index = if self.usize() == 0 {
            slice.len()
        } else {
            self.usize()
        };
        Self::new(index - 1)
    }

    pub fn usize(&self) -> usize {
        self.0.get() - 1
    }
}

// #[derive(Copy, Clone)] does not work where type parameters are not copy/clone
// https://github.com/rust-lang/rust/issues/26925
impl<T> Clone for Idx<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), Default::default())
    }
}

impl<T> Copy for Idx<T> { }

impl<T> PartialEq for Idx<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Eq for Idx<T> { }

impl<T> std::ops::Add for Idx<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        self + rhs.usize()
    }
}

impl<T> std::ops::Add<usize> for Idx<T> {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Idx::new(self.usize() + rhs)
    }
}

impl<T> std::ops::Sub for Idx<T> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self - rhs.usize()
    }
}

impl<T> std::ops::Sub<usize> for Idx<T> {
    type Output = Self;

    fn sub(self, rhs: usize) -> Self::Output {
        let index = self.usize().checked_sub(rhs).expect("Index underflow");
        Idx::new(index)
    }
}

impl<T> std::cmp::PartialEq<usize> for Idx<T> {
    fn eq(&self, other: &usize) -> bool {
        &self.usize() == other
    }
}

impl<T> std::cmp::PartialEq<Idx<T>> for usize {
    fn eq(&self, other: &Idx<T>) -> bool {
        self == &other.usize()
    }
}


impl<T> std::ops::Index<Idx<T>> for Vec<T> {
    type Output = T;

    fn index(&self, index: Idx<T>) -> &Self::Output {
        &self[index.usize()]
    }
}

impl<T> std::ops::IndexMut<Idx<T>> for Vec<T> {
    fn index_mut(&mut self, index: Idx<T>) -> &mut Self::Output {
        &mut self[index.usize()]
    }
}

impl<T> std::ops::Index<Idx<T>> for [T] {
    type Output = T;

    fn index(&self, index: Idx<T>) -> &Self::Output {
        &self[index.usize()]
    }
}

impl<T> std::ops::IndexMut<Idx<T>> for [T] {
    fn index_mut(&mut self, index: Idx<T>) -> &mut Self::Output {
        &mut self[index.usize()]
    }
}

pub trait SliceExt<T> {
    fn iter_index(&self) -> SliceIndexIter<T>;
}

pub trait VecExt<T> : SliceExt<T> {
    fn push_get_index(&mut self, value: T) -> Idx<T>;

    fn next_index(&self) -> Idx<T>;
}

impl<T> SliceExt<T> for [T] {
    fn iter_index(&self) -> SliceIndexIter<T> {
        SliceIndexIter::new(self)
    }
}

impl<T> SliceExt<T> for Vec<T> {
    fn iter_index(&self) -> SliceIndexIter<T> {
        SliceIndexIter::new(&self[..])
    }
}

impl<T> VecExt<T> for Vec<T> {
    fn push_get_index(&mut self, value: T) -> Idx<T> {
        let index = Idx::new(self.len());
        self.push(value);
        index
    }

    fn next_index(&self) -> Idx<T> {
        Idx::new(self.len())
    }
}

pub struct SliceIndexIter<'a, T> {
    slice: &'a [T],
    index: usize,
}

impl<'a, T> SliceIndexIter<'a, T> {
    fn new(slice: &'a [T]) -> Self {
        Self {
            slice,
            index: 0,
        }
    }
}

impl<'a, T> Iterator for SliceIndexIter<'a, T> {
    type Item = Idx<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.slice.len() {
            let result = Some(Idx::new(self.index));
            self.index += 1;
            result
        } else {
            None
        }
    }
}
