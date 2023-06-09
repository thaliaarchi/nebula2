// Copyright (C) 2022 Thalia Archibald
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

use crate::syntax::VariantIndex;
use crate::text::EncodingError;

pub trait Lexer = Iterator<Item = Result<Token, EncodingError>>;

/// Lexical tokens for Whitespace.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Token {
    /// Space
    S,
    /// Tab
    T,
    /// Line feed
    L,
}

impl const VariantIndex for Token {
    const COUNT: u32 = 3;
    #[inline]
    fn variant(index: u32) -> Self {
        unsafe { mem::transmute(index as u8) }
    }
    #[inline]
    fn index(&self) -> u32 {
        *self as u32
    }
}
