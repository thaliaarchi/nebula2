// Copyright (C) 2022 Thalia Archibald
//
// Nebula 2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with Nebula 2. If not, see http://www.gnu.org/licenses/.

use std::fmt::{self, Debug, Formatter};
use std::iter::FusedIterator;
use std::marker::PhantomData;
use std::ops::Range;

pub trait VariantIndex {
    const COUNT: u32;

    #[must_use]
    fn variant(index: u32) -> Self;

    #[must_use]
    fn index(&self) -> u32;

    #[must_use]
    #[inline]
    fn variant_checked(index: u32) -> Option<Self>
    where
        Self: Sized,
    {
        (index < Self::COUNT).then(|| Self::variant(index))
    }

    #[must_use]
    #[inline]
    fn iter() -> VariantRange<Self>
    where
        Self: Sized,
    {
        VariantRange::default()
    }
}

impl VariantIndex for ! {
    const COUNT: u32 = 0;
    #[inline]
    fn variant(_index: u32) -> Self {
        unreachable!()
    }
    #[inline]
    fn index(&self) -> u32 {
        *self
    }
}

pub struct VariantRange<T> {
    range: Range<u32>,
    elem: PhantomData<T>,
}

impl<T: VariantIndex> VariantRange<T> {
    #[inline]
    pub fn new(start: &T, end: &T) -> Self {
        VariantRange {
            range: T::index(&start)..T::index(&end) + 1,
            elem: PhantomData,
        }
    }

    #[inline]
    pub fn contains(&self, variant: &T) -> bool {
        self.range.contains(&T::index(variant))
    }
}

impl<T: VariantIndex> Iterator for VariantRange<T> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.range.next().map(T::variant)
    }
    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.range.nth(n).map(T::variant)
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.range.size_hint()
    }
    #[inline]
    fn count(self) -> usize {
        self.range.count()
    }
    #[inline]
    fn last(self) -> Option<T> {
        self.range.last().map(T::variant)
    }
    #[inline]
    fn min(self) -> Option<T> {
        self.range.min().map(T::variant)
    }
    #[inline]
    fn max(self) -> Option<T> {
        self.range.max().map(T::variant)
    }
}

impl<T: VariantIndex> DoubleEndedIterator for VariantRange<T> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.range.next_back().map(T::variant)
    }
    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.range.nth_back(n).map(T::variant)
    }
}

impl<T: VariantIndex> ExactSizeIterator for VariantRange<T> {
    #[inline]
    fn len(&self) -> usize {
        self.range.len()
    }
}

impl<T: VariantIndex> FusedIterator for VariantRange<T> {}

impl<T: VariantIndex> Default for VariantRange<T> {
    #[inline]
    fn default() -> Self {
        VariantRange {
            range: 0..T::COUNT,
            elem: PhantomData,
        }
    }
}

impl<T> Clone for VariantRange<T> {
    #[must_use]
    #[inline]
    fn clone(&self) -> Self {
        VariantRange {
            range: self.range.clone(),
            elem: PhantomData,
        }
    }
}

impl<T: Debug + VariantIndex> Debug for VariantRange<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if T::COUNT == 0 {
            write!(f, "..")
        } else {
            let start = T::variant(self.range.start);
            if self.range.end == 0 || self.range.start >= self.range.end {
                write!(f, "{:?}", start..T::variant(self.range.end))
            } else {
                write!(f, "{:?}", start..=T::variant(self.range.end - 1))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ws::Token;

    macro_rules! check_debug(($left:expr, $right:expr) => {
        assert_eq!($left, format!("{:?}", $right));
    });

    #[test]
    fn iter_no_variants() {
        let mut iter = <!>::iter();
        check_debug!("..", iter);
        assert_eq!(None, iter.next());
    }

    #[test]
    fn iter_one_variant() {
        #[allow(dead_code)]
        #[derive(Debug, PartialEq, Eq)]
        enum E {
            V,
        }
        impl VariantIndex for E {
            const COUNT: u32 = 1;
            fn variant(_index: u32) -> Self {
                E::V
            }
            fn index(&self) -> u32 {
                0
            }
        }

        let mut iter = E::iter();
        check_debug!("V..=V", iter);
        assert_eq!(Some(E::V), iter.next());
        check_debug!("V..V", iter);
        iter = E::iter();
        assert_eq!(Some(E::V), iter.next_back());
        check_debug!("V..V", iter);
    }

    #[test]
    fn iter_three_variants() {
        let mut iter = Token::iter();
        check_debug!("S..=L", iter);
        assert_eq!(Some(Token::S), iter.next());
        check_debug!("T..=L", iter);
        assert_eq!(Some(Token::L), iter.next_back());
        check_debug!("T..=T", iter);
        assert_eq!(Some(Token::T), iter.next());
        check_debug!("L..L", iter);
        assert_eq!(None, iter.next());
    }

    #[test]
    fn iter_out_of_order() {
        let mut iter = VariantRange::new(&Token::L, &Token::S);
        check_debug!("L..T", iter);
        assert_eq!(None, iter.next());
    }
}
