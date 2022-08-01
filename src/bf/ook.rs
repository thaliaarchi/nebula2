// Copyright (C) 2022 Andrew Archibald
//
// Nebula 2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with Nebula 2. If not, see http://www.gnu.org/licenses/.

use std::mem;
use std::sync::LazyLock;

use crate::bf;
use crate::syntax::{EnumIndex, PrefixTable};

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

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Inst {
    Bf(bf::Inst),
    /// `Ook? Ook?`
    Banana,
}

pub static TABLE: LazyLock<PrefixTable<Token, Inst>> = LazyLock::new(|| {
    use bf::Inst::*;
    let mut table = PrefixTable::with_dense_width(2);
    table.insert(&[T![.], T![?]], Right.into()).unwrap();
    table.insert(&[T![?], T![.]], Left.into()).unwrap();
    table.insert(&[T![.], T![.]], Inc.into()).unwrap();
    table.insert(&[T![!], T![!]], Dec.into()).unwrap();
    table.insert(&[T![!], T![.]], Output.into()).unwrap();
    table.insert(&[T![.], T![!]], Input.into()).unwrap();
    table.insert(&[T![!], T![?]], Head.into()).unwrap();
    table.insert(&[T![?], T![!]], Tail.into()).unwrap();
    table.insert(&[T![?], T![?]], Inst::Banana).unwrap();
    table
});

impl const From<bf::Inst> for Inst {
    fn from(inst: bf::Inst) -> Self {
        Inst::Bf(inst)
    }
}

impl const EnumIndex for Token {
    const COUNT: u32 = 3;

    fn from_index(index: u32) -> Self {
        unsafe { mem::transmute(index as u8) }
    }

    fn to_index(&self) -> u32 {
        *self as u32
    }
}
