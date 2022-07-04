// Copyright (C) 2022 Andrew Archibald
//
// yspace2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with yspace2. If not, see http://www.gnu.org/licenses/.

use std::fmt::{self, Debug, Formatter};

use crate::ws::token::Token;

const LEN_BITS: u64 = 6;
const LEN_MASK: u64 = 0b111111;

#[derive(Clone, Copy, Default)]
pub struct TokenVec(u64);

impl TokenVec {
    #[inline]
    pub const fn new() -> Self {
        TokenVec(0)
    }

    #[inline]
    pub fn push(&mut self, tok: Token) {
        TokenVec(self.0) = self.push_const(tok);
    }

    #[inline]
    pub fn pop(&mut self) -> Token {
        let (tok, vec) = self.pop_const();
        self.0 = vec.0;
        tok
    }

    #[inline]
    pub fn push_front(&mut self, tok: Token) {
        TokenVec(self.0) = self.push_front_const(tok);
    }

    #[inline]
    pub fn pop_front(&mut self) -> Token {
        let (tok, vec) = self.pop_front_const();
        self.0 = vec.0;
        tok
    }

    #[inline]
    pub const fn push_const(&self, tok: Token) -> TokenVec {
        let len = self.len();
        self.set_len_const(len + 1).set_const(len, tok)
    }

    #[inline]
    pub const fn pop_const(&self) -> (Token, TokenVec) {
        let len = self.len();
        (self.get(len), self.set_len_const(len - 1))
    }

    #[inline]
    pub const fn push_front_const(&self, tok: Token) -> TokenVec {
        debug_assert!(self.len() < self.cap());
        TokenVec((self.0 & !LEN_MASK) << 2 | (tok as u64) << LEN_BITS | (self.0 & LEN_MASK) + 1)
    }

    #[inline]
    pub const fn pop_front_const(&self) -> (Token, TokenVec) {
        (
            self.get(0),
            TokenVec(self.0 >> 2).set_len_const(self.len() - 1),
        )
    }

    #[inline]
    pub const fn get(&self, i: usize) -> Token {
        debug_assert!(i < self.len());
        match (self.0 >> Self::shift_for(i)) & 0b11 {
            0 => Token::S,
            1 => Token::T,
            2 => Token::L,
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn set(&mut self, i: usize, tok: Token) {
        TokenVec(self.0) = self.set_const(i, tok);
    }

    #[inline]
    pub const fn set_const(&self, i: usize, tok: Token) -> TokenVec {
        debug_assert!(i < self.len());
        let shift = Self::shift_for(i);
        TokenVec(self.0 & !(0b11 << shift) | ((tok as u64) << shift))
    }

    #[inline]
    pub const fn len(&self) -> usize {
        (self.0 & LEN_MASK) as usize
    }

    #[inline]
    const fn set_len_const(&self, len: usize) -> TokenVec {
        debug_assert!(len <= self.cap());
        TokenVec((self.0 & !LEN_MASK) | len as u64)
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
}

impl Iterator for TokenVec {
    type Item = Token;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        (!self.empty()).then(|| self.pop_front())
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
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

#[macro_export]
macro_rules! token_vec[
    (@tok S) => { $crate::ws::token::Token::S };
    (@tok T) => { $crate::ws::token::Token::T };
    (@tok L) => { $crate::ws::token::Token::L };
    (@tok $tok:expr) => { $tok };
    ($($tok:tt)*) => {
        TokenVec(0)$(.push_const(token_vec!(@tok $tok)))*
    };
];
