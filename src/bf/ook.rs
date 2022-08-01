// Copyright (C) 2022 Andrew Archibald
//
// Nebula 2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with Nebula 2. If not, see http://www.gnu.org/licenses/.

use std::mem;
use std::sync::LazyLock;

use crate::bf::Inst;
use crate::syntax::{EnumIndex, PrefixTable, TokenSeq};

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

pub static TABLE: LazyLock<PrefixTable<Token, Inst>> = LazyLock::new(|| {
    let dense_len = TokenSeq::from(&[T![!], T![!]]).as_usize() + 1;
    let mut table = PrefixTable::new(dense_len);
    table.insert(&[T![.], T![?]], Inst::Right).unwrap();
    table.insert(&[T![?], T![.]], Inst::Left).unwrap();
    table.insert(&[T![.], T![.]], Inst::Inc).unwrap();
    table.insert(&[T![!], T![!]], Inst::Dec).unwrap();
    table.insert(&[T![!], T![.]], Inst::Output).unwrap();
    table.insert(&[T![.], T![!]], Inst::Input).unwrap();
    table.insert(&[T![!], T![?]], Inst::Head).unwrap();
    table.insert(&[T![?], T![!]], Inst::Tail).unwrap();
    table.insert(&[T![?], T![?]], Inst::Banana).unwrap();
    table
});

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
