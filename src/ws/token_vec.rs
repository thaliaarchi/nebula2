// Copyright (C) 2022 Andrew Archibald
//
// yspace2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with yspace2. If not, see http://www.gnu.org/licenses/.

use crate::ws::token::Token;

#[derive(Clone, Copy, Default)]
pub struct TokenVec {
    data: u64,
}

impl TokenVec {
    const LEN_BITS: u64 = 6;
    const LEN_MASK: u64 = 0b111111;

    #[inline]
    pub const fn new() -> Self {
        TokenVec { data: 0 }
    }

    #[inline]
    pub fn push(&mut self, tok: Token) {
        let len = self.len();
        self.set_len(len + 1);
        self.set(len, tok);
    }

    #[inline]
    pub fn pop(&mut self) -> Token {
        let len = self.len();
        self.set_len(len - 1);
        self.get(len)
    }

    #[inline]
    pub fn push_front(&mut self, tok: Token) {
        debug_assert!(self.len() < self.cap());
        self.data = (self.data & !Self::LEN_MASK) << 2
            | (self.data & Self::LEN_MASK) + 1
            | (tok as u64) << Self::LEN_BITS;
    }

    #[inline]
    pub fn pop_front(&mut self) -> Token {
        let tok = self.get(0);
        self.data = (self.data >> 2) & !Self::LEN_MASK | (self.data & Self::LEN_MASK) - 1;
        tok
    }

    #[inline]
    pub const fn get(&self, i: usize) -> Token {
        debug_assert!(i < self.len());
        match (self.data >> Self::shift_for(i)) & 0b11 {
            0 => Token::S,
            1 => Token::T,
            2 => Token::L,
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn set(&mut self, i: usize, tok: Token) {
        debug_assert!(i < self.len());
        let shift = Self::shift_for(i);
        self.data = self.data & !(0b11 << shift) | ((tok as u64) << shift);
    }

    #[inline]
    pub const fn len(&self) -> usize {
        (self.data & Self::LEN_MASK) as usize
    }

    #[inline]
    fn set_len(&mut self, len: usize) {
        debug_assert!(len <= self.cap());
        self.data = (self.data & !Self::LEN_MASK) | len as u64;
    }

    #[inline]
    pub const fn cap(&self) -> usize {
        29
    }

    #[inline]
    const fn shift_for(i: usize) -> u64 {
        i as u64 * 2 + Self::LEN_BITS
    }
}

#[macro_export]
macro_rules! token_vec[
    ($($tok:ident)*) => {
        TokenVec {
            data: (token_vec!(@concat $($tok)*)) << 6
                | (0 $(+ token_vec!(@one $tok))+)
        }
    };
    (@concat $tok:ident $($rest:ident)*) => {
        token_vec!(@tok $tok) | (token_vec!(@concat $($rest)*)) << 2
    };
    (@concat) => { 0 };
    (@tok S) => { 0 };
    (@tok T) => { 1 };
    (@tok L) => { 2 };
    (@one $tok:ident) => { 1 };
];
