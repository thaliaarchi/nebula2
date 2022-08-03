// Copyright (C) 2022 Andrew Archibald
//
// Nebula 2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with Nebula 2. If not, see http://www.gnu.org/licenses/.

use std::fmt::{self, Debug, Formatter};
use std::iter::FusedIterator;
use std::marker::PhantomData;

pub trait VariantIndex {
    const COUNT: u32;

    fn variant(index: u32) -> Self;
    fn index(&self) -> u32;

    fn iter() -> VariantIter<Self>
    where
        Self: Sized,
    {
        VariantIter::default()
    }
}

pub struct VariantIter<T> {
    front: u32,
    back: u32,
    elem: PhantomData<T>,
}

impl<T: VariantIndex> Iterator for VariantIter<T> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.nth(0)
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let front = self.front + n as u32;
        if front < self.back {
            self.front = front + 1;
            Some(T::variant(front))
        } else {
            self.front = self.back;
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let s = (self.back - self.front) as usize;
        (s, Some(s))
    }
}

impl<T: VariantIndex> DoubleEndedIterator for VariantIter<T> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.nth_back(0)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let n = n as u32;
        if self.front + n < self.back {
            self.back = self.back - n - 1;
            Some(T::variant(self.back))
        } else {
            self.back = self.front;
            None
        }
    }
}

impl<T: VariantIndex> ExactSizeIterator for VariantIter<T> {
    #[inline]
    fn len(&self) -> usize {
        self.size_hint().0
    }
}

impl<T: VariantIndex> const FusedIterator for VariantIter<T> {}

impl<T: VariantIndex> const Default for VariantIter<T> {
    #[inline]
    fn default() -> Self {
        VariantIter {
            front: 0,
            back: T::COUNT,
            elem: PhantomData,
        }
    }
}

// Avoid extra bounds for T from derive
impl<T> const Clone for VariantIter<T> {
    fn clone(&self) -> Self {
        VariantIter {
            front: self.front,
            back: self.back,
            elem: PhantomData,
        }
    }
}
impl<T> const Copy for VariantIter<T> {}
impl<T> Debug for VariantIter<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("VariantIter")
            .field("front", &self.front)
            .field("back", &self.back)
            .finish()
    }
}
