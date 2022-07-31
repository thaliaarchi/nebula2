// Copyright (C) 2022 Andrew Archibald
//
// Nebula 2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with Nebula 2. If not, see http://www.gnu.org/licenses/.

use std::mem;

use crate::syntax::{FromRepr, PrefixTable, TokenSeq};

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Token {
    Period,
    Question,
    Bang,
}

macro_rules! T[
    (.) => { Token::Period };
    (?) => { Token::Question };
    (!) => { Token::Bang };
];

pub fn parser() -> PrefixTable<Token> {
    let dense_len = TokenSeq::from(&[T![!], T![!]]).as_usize() + 1;
    let table = PrefixTable::new(dense_len);
    // table.register(&[T![.], T![?]], Inst::Right).unwrap();
    // table.register(&[T![?], T![.]], Inst::Left).unwrap();
    // table.register(&[T![.], T![.]], Inst::Inc).unwrap();
    // table.register(&[T![!], T![!]], Inst::Dec).unwrap();
    // table.register(&[T![!], T![.]], Inst::Output).unwrap();
    // table.register(&[T![.], T![!]], Inst::Input).unwrap();
    // table.register(&[T![!], T![?]], Inst::Head).unwrap();
    // table.register(&[T![?], T![!]], Inst::Tail).unwrap();
    // table.register(&[T![?], T![?]], Inst::Nop).unwrap();
    table
}

impl const FromRepr for Token {
    const MAX: u32 = 3;

    #[inline]
    fn repr(&self) -> u32 {
        *self as u32
    }

    #[inline]
    fn try_from_repr(v: u32) -> Option<Self> {
        if v < Self::MAX {
            Some(unsafe { Self::from_repr_unchecked(v) })
        } else {
            None
        }
    }

    #[inline]
    unsafe fn from_repr_unchecked(v: u32) -> Self {
        mem::transmute(v as u8)
    }
}
