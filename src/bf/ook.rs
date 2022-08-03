// Copyright (C) 2022 Andrew Archibald
//
// Nebula 2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with Nebula 2. If not, see http://www.gnu.org/licenses/.

//! [Ook!](https://www.dangermouse.net/esoteric/ook.html) language syntax.

use std::mem;
use std::sync::LazyLock;

use crate::bf;
use crate::syntax::{PrefixTable, VariantIndex};

/// Punctuation tokens used in Ook! syntax.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Punct {
    /// `Ook.`
    Period,
    /// `Ook?`
    Question,
    /// `Ook!`
    Bang,
}

macro_rules! P[
    (.) => { Punct::Period };
    (?) => { Punct::Question };
    (!) => { Punct::Bang };
];

/// Ook! instructions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Inst {
    /// Ook! instructions that are isomorphic to Brainfuck instructions.
    ///
    /// - `Ook. Ook?` — `>`
    /// - `Ook? Ook.` — `<`
    /// - `Ook. Ook.` — `+`
    /// - `Ook! Ook!` — `-`
    /// - `Ook. Ook!` — `,`
    /// - `Ook! Ook.` — `.`
    /// - `Ook! Ook?` — `[`
    /// - `Ook? Ook!` — `]`
    Bf(bf::Inst),
    /// Extension instruction.
    ///
    /// - `Ook? Ook?` — Give the memory pointer a banana.
    ///
    /// Its semantics are intentionally left ambiguous, according to the author
    /// via email:
    ///
    /// > Well honestly it's just another joke in what's already a joke
    /// > language. Feel free to interpret/implement it any way you feel makes
    /// > the most sense or the most fun. :-)
    ///
    /// It was added to the Ook! specification on [2022-03-07](https://web.archive.org/web/20220424180804/https://www.dangermouse.net/esoteric/ook.html)
    /// and is rarely supported by implementations, instead usually being
    /// treated as a parse error.
    Banana,
}

static_assertions::assert_eq_size!(Inst, Option<Inst>, u8);

/// Prefix table for parsing Ook! punctuation.
pub static TABLE: LazyLock<PrefixTable<Punct, Inst>> = LazyLock::new(|| {
    use bf::Inst::*;
    let mut table = PrefixTable::with_dense_width(2);
    table.insert(&[P![.], P![?]], Right.into()).unwrap();
    table.insert(&[P![?], P![.]], Left.into()).unwrap();
    table.insert(&[P![.], P![.]], Inc.into()).unwrap();
    table.insert(&[P![!], P![!]], Dec.into()).unwrap();
    table.insert(&[P![.], P![!]], Input.into()).unwrap();
    table.insert(&[P![!], P![.]], Output.into()).unwrap();
    table.insert(&[P![!], P![?]], Head.into()).unwrap();
    table.insert(&[P![?], P![!]], Tail.into()).unwrap();
    table.insert(&[P![?], P![?]], Inst::Banana).unwrap();
    table
});

impl const From<bf::Inst> for Inst {
    fn from(inst: bf::Inst) -> Self {
        Inst::Bf(inst)
    }
}

impl const VariantIndex for Punct {
    const COUNT: u32 = 3;

    fn variant(index: u32) -> Self {
        unsafe { mem::transmute(index as u8) }
    }

    fn index(&self) -> u32 {
        *self as u32
    }
}
