// Copyright (C) 2022 Andrew Archibald
//
// yspace2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with yspace2. If not, see http://www.gnu.org/licenses/.

use std::fmt::{self, Debug, Formatter};

use crate::ws::token::{Token, TokenSeq};

const LEN_BITS: u64 = 6;
const LEN_MASK: u64 = 0b111111;

#[derive(Clone, Copy, Default, Eq)]
pub struct TokenVec(u64);

impl TokenVec {
    #[inline]
    pub const fn new() -> Self {
        TokenVec(0)
    }

    #[inline]
    pub const fn push(&mut self, tok: Token) {
        let len = self.len();
        self.set_len(len + 1);
        self.set(len, tok);
    }

    #[inline]
    pub const fn pop(&mut self) -> Token {
        let len = self.len() - 1;
        let tok = self.get(len);
        self.set_len(len);
        tok
    }

    #[inline]
    pub const fn push_front(&mut self, tok: Token) {
        debug_assert!(self.len() < self.cap());
        let data = (self.0 & !LEN_MASK) << 2;
        let len = (self.0 & LEN_MASK) + 1;
        self.0 = data | (tok as u64) << LEN_BITS | len;
    }

    #[inline]
    pub const fn pop_front(&mut self) -> Token {
        let tok = self.get(0);
        let len = self.len();
        self.0 >>= 2;
        self.set_len(len - 1);
        tok
    }

    #[inline]
    pub const fn get(&self, i: usize) -> Token {
        debug_assert!(i < self.len());
        unsafe { Token::from_unchecked(((self.0 >> Self::shift_for(i)) & 0b11) as u8) }
    }

    #[inline]
    pub const fn set(&mut self, i: usize, tok: Token) {
        debug_assert!(i < self.len());
        let shift = Self::shift_for(i);
        self.0 = self.0 & !(0b11 << shift) | ((tok as u64) << shift);
    }

    #[inline]
    pub const fn concat(&mut self, other: &TokenVec) {
        let shift = Self::shift_for(self.len());
        self.extend(other.len());
        self.0 |= (other.0 & !LEN_MASK) << (shift - LEN_BITS);
    }

    #[inline]
    pub const fn append(&mut self, toks: &[Token]) {
        let shift = Self::shift_for(self.len());
        self.extend(toks.len());
        self.0 |= Self::bits(toks) << shift;
    }

    #[inline]
    pub const fn append_front(&mut self, toks: &[Token]) {
        self.extend_front(toks.len());
        self.0 |= Self::bits(toks) << LEN_BITS;
    }

    #[inline]
    const fn extend(&mut self, n: usize) {
        // Shift overflows if n == 0
        debug_assert!(n != 0 && self.len() + n <= self.cap());
        let data = self.0 & ((1 << Self::shift_for(self.len())) - 1);
        let len = (self.0 & LEN_MASK) + n as u64;
        self.0 = data | len;
    }

    #[inline]
    const fn extend_front(&mut self, n: usize) {
        debug_assert!(self.len() + n <= self.cap());
        let data = (self.0 & !LEN_MASK) << Self::shift_for(n);
        let len = (self.0 & LEN_MASK) + n as u64;
        self.0 = data | len;
    }

    #[inline]
    pub const fn len(&self) -> usize {
        (self.0 & LEN_MASK) as usize
    }

    #[inline]
    const fn set_len(&mut self, len: usize) {
        debug_assert!(len <= self.cap());
        self.0 = (self.0 & !LEN_MASK) | len as u64;
    }

    #[inline]
    pub const fn cap(&self) -> usize {
        29
    }

    #[inline]
    pub const fn empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    const fn shift_for(i: usize) -> u64 {
        i as u64 * 2 + LEN_BITS
    }

    #[inline]
    const fn bits(toks: &[Token]) -> u64 {
        let mut bits = 0;
        let mut i = 0;
        while i < toks.len() {
            bits |= (toks[i] as u64) << i * 2;
            i += 1;
        }
        bits
    }
}

impl const From<&[Token]> for TokenVec {
    #[inline]
    fn from(toks: &[Token]) -> Self {
        TokenVec(Self::bits(toks) << LEN_BITS | toks.len() as u64)
    }
}

impl<const N: usize> const From<&[Token; N]> for TokenVec {
    #[inline]
    fn from(toks: &[Token; N]) -> Self {
        TokenVec(Self::bits(toks) << LEN_BITS | N as u64)
    }
}

impl const From<TokenSeq> for TokenVec {
    #[inline]
    fn from(seq: TokenSeq) -> TokenVec {
        let mut seq = seq;
        let mut toks = TokenVec::new();
        let mut i = seq.len();
        while i != 0 {
            i -= 1;
            toks.push_front(seq.pop());
        }
        toks
    }
}

impl const Iterator for TokenVec {
    type Item = Token;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if !self.empty() {
            Some(self.pop_front())
        } else {
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl const DoubleEndedIterator for TokenVec {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if !self.empty() {
            Some(self.pop())
        } else {
            None
        }
    }
}

impl Debug for TokenVec {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("[")?;
        let mut first = true;
        for tok in *self {
            if !first {
                f.write_str(" ")?;
            }
            write!(f, "{:?}", tok)?;
            first = false;
        }
        f.write_str("]")?;
        Ok(())
    }
}

impl const PartialEq for TokenVec {
    #[inline]
    fn eq(&self, other: &TokenVec) -> bool {
        // Shift overflows if len == cap
        debug_assert!(self.len() < self.cap());
        let (len1, len2) = (self.len(), other.len());
        (self.0 << Self::shift_for(len1)) == (other.0 << Self::shift_for(len2))
    }
}

macro_rules! token_vec[
    (@tok S) => { $crate::ws::token::Token::S };
    (@tok T) => { $crate::ws::token::Token::T };
    (@tok L) => { $crate::ws::token::Token::L };
    (@tok $tok:expr) => { $tok };
    () => { $crate::ws::token_vec::TokenVec::new() };
    ($($tok:tt)+) => {
        $crate::ws::token_vec::TokenVec::from(&[$(token_vec!(@tok $tok)),+])
    };
];
pub(crate) use token_vec;
