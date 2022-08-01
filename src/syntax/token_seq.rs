// Copyright (C) 2022 Andrew Archibald
//
// Nebula 2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with Nebula 2. If not, see http://www.gnu.org/licenses/.

use std::fmt::{self, Debug, Formatter};
use std::marker::PhantomData;

// Maximum TokenSeq value for each integer width:
// - u8  [T T L L L]
// - u16 [T L T T T S T L S L]
// - u32 [L S T L S L T T S L S T T S S S S S L L]
// - u64 [L S T S S T L L S L L S L S S L S L L L S S L L T S S T T S T T L T S T T S S S S]
#[repr(transparent)]
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TokenSeq<T> {
    inner: u32,
    elem: PhantomData<T>,
}

impl<T: FromRepr> TokenSeq<T> {
    #[inline]
    pub const fn new() -> Self {
        TokenSeq { inner: 0, elem: PhantomData }
    }

    #[inline]
    pub fn push(&mut self, tok: T) {
        self.inner = self.inner * T::MAX + tok.repr() + 1;
    }

    #[inline]
    pub fn pop(&mut self) -> T {
        let tok = unsafe { T::from_repr_unchecked((self.inner - 1) % T::MAX) };
        self.inner = (self.inner - 1) / T::MAX;
        tok
    }

    #[inline]
    pub const fn len(&self) -> u32 {
        let mut seq = self.inner;
        let mut len = 0;
        while seq != 0 {
            seq = (seq - 1) / T::MAX;
            len += 1;
        }
        len
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.inner == 0
    }

    #[inline]
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
        TokenSeq {
            inner: seq as u32,
            elem: PhantomData,
        }
    }
}

impl<T: Copy + FromRepr> From<&[T]> for TokenSeq<T> {
    fn from(toks: &[T]) -> Self {
        let mut seq = TokenSeq::new();
        for &tok in toks {
            seq.push(tok);
        }
        seq
    }
}

impl<T: Copy + FromRepr, const N: usize> From<&[T; N]> for TokenSeq<T> {
    fn from(toks: &[T; N]) -> Self {
        TokenSeq::from(toks.as_slice())
    }
}

impl<T: FromRepr> From<TokenSeq<T>> for Vec<T> {
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

impl<T: Copy + Debug + FromRepr> Debug for TokenSeq<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("TokenSeq")
            .field(&self.inner)
            .field(&Vec::from(*self))
            .finish()
    }
}

pub trait FromRepr
where
    Self: Sized,
{
    const MAX: u32;

    fn repr(&self) -> u32;
    fn try_from_repr(v: u32) -> Option<Self>;
    unsafe fn from_repr_unchecked(v: u32) -> Self;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ws::token::{token_vec, TokenVec};

    #[test]
    fn convert() {
        macro_rules! token_vecs(
            ($([$($seq:tt)*]),+$(,)?) => { vec![$(token_vec![$($seq)*]),+] }
        );
        let seqs: Vec<TokenVec> = token_vecs![
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
            let toks2 = TokenVec::from(seq);
            assert_eq!(toks, toks2, "TokenVec::from({seq:?})");
        }
    }
}
