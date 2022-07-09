// Copyright (C) 2022 Andrew Archibald
//
// Nebula 2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with Nebula 2. If not, see http://www.gnu.org/licenses/.

pub use bit_pack::*;
pub use mapping::*;
pub(crate) use token_seq::*;
pub use token_vec::*;

mod bit_pack;
mod mapping;
mod token_seq;
mod token_vec;

use std::mem;

use crate::text::EncodingError;

pub trait Lexer = Iterator<Item = Result<Token, EncodingError>>;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Token {
    S = 0,
    T = 1,
    L = 2,
}

impl Token {
    #[inline]
    pub const unsafe fn from_unchecked(n: u8) -> Self {
        mem::transmute(n)
    }
}
