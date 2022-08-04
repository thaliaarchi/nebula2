// Copyright (C) 2022 Andrew Archibald
//
// Nebula 2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with Nebula 2. If not, see http://www.gnu.org/licenses/.

//! Brainfuck language.
//!
//! # Resources
//!
//! - [Original distribution](http://main.aminet.net/dev/lang/brainfuck-2.lha)
//! - [Esolang wiki](https://esolangs.org/wiki/Brainfuck)

use std::mem;

use crate::syntax::VariantIndex;

pub mod ook;
pub mod spoon;

/// Brainfuck instructions.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Inst {
    /// `>`
    Right,
    /// `<`
    Left,
    /// `+`
    Inc,
    /// `-`
    Dec,
    /// `.`
    Output,
    /// `,`
    Input,
    /// `[`
    Head,
    /// `]`
    Tail,
}

/// Brainfuck instructions with debug extension.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InstExt {
    /// Standard instructions.
    Bf(Inst),
    /// `#` extension instruction from `bfi.c` in the original distribution.
    Debug,
}

impl const VariantIndex for Inst {
    const COUNT: u32 = 8;
    #[inline]
    fn variant(index: u32) -> Self {
        unsafe { mem::transmute(index as u8) }
    }
    #[inline]
    fn index(&self) -> u32 {
        *self as u32
    }
}

impl const VariantIndex for InstExt {
    const COUNT: u32 = 9;
    #[inline]
    fn variant(index: u32) -> Self {
        unsafe { mem::transmute(index as u8) }
    }
    #[inline]
    fn index(&self) -> u32 {
        unsafe { mem::transmute::<_, u8>(*self) as u32 }
    }
}
