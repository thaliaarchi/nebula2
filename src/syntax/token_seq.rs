// Copyright (C) 2022 Andrew Archibald
//
// Nebula 2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with Nebula 2. If not, see http://www.gnu.org/licenses/.

use std::cmp::Ordering;
use std::fmt::{self, Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

use crate::syntax::VariantIndex;

/// A compact token stack, that represents a sequence of tokens as a scalar.
///
/// `TokenSeq` can be used to order or hash token sequences. For example, with
/// [`ws::Token`](crate::ws::Token), which has 3 variants—`S`, `T`, and `L`—,
/// `[]` is represented by a `TokenSeq` of 0, `[S]` is 1, `[T]` is 2, `[L]` is
/// 3, `[S S]` is 4, `[S T]` is 5, etc.
///
/// # Capacity
///
/// The capacity differs by the number of variants that `T` has (i.e.,
/// [`<T as VariantIndex>::COUNT`](VariantIndex::COUNT)) and can be calculated
/// with `TokenSeq::<T>::MAX.len()`. For convenience, the ceiling of the
/// capacity for common sizes is:
///
/// - 2 variants => capacity 32
/// - 3 variants => capacity 20
/// - 4 variants => capacity 16
/// - 5 variants => capacity 14
/// - 6 variants => capacity 13
/// - 7 variants => capacity 12
/// - 8..=9 variants => capacity 11
/// - 10..=11 variants => capacity 10
/// - 12..=15 variants => capacity 9
/// - 16..=23 variants => capacity 8
/// - 24..=40 variants => capacity 7
/// - 41..=84 variants => capacity 6
/// - 85..=255 variants => capacity 5
/// - …
#[repr(transparent)]
pub struct TokenSeq<T> {
    inner: u32,
    elem: PhantomData<T>,
}

impl<T: VariantIndex> TokenSeq<T> {
    pub const MAX: TokenSeq<T> = TokenSeq::from(u32::MAX);

    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        TokenSeq { inner: 0, elem: PhantomData }
    }

    #[inline]
    #[must_use]
    pub const fn size_for(width: usize) -> usize {
        let mut i = width;
        let mut max = 0;
        while i > 0 {
            max = max * T::COUNT + T::COUNT;
            i -= 1;
        }
        max as usize + 1
    }

    #[inline]
    pub fn push(&mut self, tok: &T) {
        let v = tok.index();
        debug_assert!(v < T::COUNT);
        self.inner = self.inner * T::COUNT + v + 1;
    }

    #[inline]
    pub fn pop(&mut self) -> T {
        let v = (self.inner - 1) % T::COUNT;
        debug_assert!(v < T::COUNT);
        let tok = T::variant(v);
        self.inner = (self.inner - 1) / T::COUNT;
        tok
    }

    #[inline]
    #[must_use]
    pub const fn len(&self) -> u32 {
        let mut seq = self.inner;
        let mut len = 0;
        while seq != 0 {
            seq = (seq - 1) / T::COUNT;
            len += 1;
        }
        len
    }

    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.inner == 0
    }

    #[inline]
    #[must_use]
    pub const fn as_usize(&self) -> usize {
        self.inner as usize
    }
}

impl<T> const From<u32> for TokenSeq<T> {
    #[inline]
    fn from(seq: u32) -> Self {
        TokenSeq { inner: seq, elem: PhantomData }
    }
}

impl<T> const From<usize> for TokenSeq<T> {
    #[inline]
    fn from(seq: usize) -> Self {
        debug_assert!(seq < u32::MAX as usize);
        TokenSeq {
            inner: seq as u32,
            elem: PhantomData,
        }
    }
}

impl<T: VariantIndex> From<&[T]> for TokenSeq<T> {
    fn from(toks: &[T]) -> Self {
        let mut seq = TokenSeq::new();
        for tok in toks {
            seq.push(tok);
        }
        seq
    }
}

impl<T: VariantIndex, const N: usize> From<&[T; N]> for TokenSeq<T> {
    fn from(toks: &[T; N]) -> Self {
        TokenSeq::from(toks.as_slice())
    }
}

impl<T: VariantIndex> From<TokenSeq<T>> for Vec<T> {
    fn from(seq: TokenSeq<T>) -> Vec<T> {
        let mut seq = seq;
        let mut toks = Vec::new();
        while !seq.is_empty() {
            toks.push(seq.pop());
        }
        toks.reverse();
        toks
    }
}

impl<T: Debug + VariantIndex> Debug for TokenSeq<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("TokenSeq")
            .field(&self.inner)
            .field(&Vec::from(*self))
            .finish()
    }
}

// Avoid extra bounds for T from derive
impl<T> const Clone for TokenSeq<T> {
    fn clone(&self) -> Self {
        TokenSeq {
            inner: self.inner,
            elem: PhantomData,
        }
    }
}
impl<T> const Copy for TokenSeq<T> {}
impl<T: VariantIndex> const Default for TokenSeq<T> {
    fn default() -> Self {
        TokenSeq::new()
    }
}
impl<T> const PartialEq for TokenSeq<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}
impl<T> const Eq for TokenSeq<T> {}
impl<T> PartialOrd for TokenSeq<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.inner.partial_cmp(&other.inner)
    }
}
impl<T> Ord for TokenSeq<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.inner.cmp(&other.inner)
    }
}
impl<T> Hash for TokenSeq<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ws::token::{Token, Token::*};

    #[test]
    fn convert() {
        macro_rules! tokens(
            ($([$($seq:tt)*]),+$(,)?) => { vec![$(&[$($seq),*]),+] }
        );
        let seqs: Vec<&[Token]> = tokens![
            [],
            [S], [T], [L],
            [S S], [S T], [S L],
            [T S], [T T], [T L],
            [L S], [L T], [L L],
            [S S S], [S S T], [S S L], [S T S], [S T T], [S T L], [S L S], [S L T], [S L L],
            [T S S], [T S T], [T S L], [T T S], [T T T], [T T L], [T L S], [T L T], [T L L],
            [L S S], [L S T], [L S L], [L T S], [L T T], [L T L], [L L S], [L L T], [L L L],
        ];
        for (i, &toks) in seqs.iter().enumerate() {
            let seq = TokenSeq::from(i);
            let seq2 = TokenSeq::from(toks);
            assert_eq!(seq, seq2, "TokenSeq::from({toks:?})");
            let toks2 = Vec::from(seq);
            assert_eq!(toks, toks2, "TokenVec::from({seq:?})");
        }
    }
}
