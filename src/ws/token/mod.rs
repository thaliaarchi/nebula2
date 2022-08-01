// Copyright (C) 2022 Andrew Archibald
//
// Nebula 2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with Nebula 2. If not, see http://www.gnu.org/licenses/.

pub use bit_pack::*;
pub use mapping::*;
pub use token_vec::*;

mod bit_pack;
mod mapping;
mod token_vec;

use std::mem;

use crate::syntax::EnumIndex;
use crate::text::EncodingError;

pub trait Lexer = Iterator<Item = Result<Token, EncodingError>>;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Token {
    S,
    T,
    L,
}

impl const EnumIndex for Token {
    const COUNT: u32 = 3;
}

impl const From<u32> for Token {
    fn from(v: u32) -> Self {
        unsafe { mem::transmute(v as u8) }
    }
}

impl const From<Token> for u32 {
    fn from(tok: Token) -> u32 {
        tok as u32
    }
}
