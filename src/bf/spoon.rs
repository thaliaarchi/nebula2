// Copyright (C) 2022 Thalia Archibald
//
// Nebula 2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with Nebula 2. If not, see http://www.gnu.org/licenses/.

//! Spoon language.
//!
//! # Resources
//!
//! - [Language specification](https://web.archive.org/web/20140228003324/http://www.bluedust.dontexist.com/spoon)
//! - [Reference interpreter mirror](http://marquisdegeek.com/pub/sources/spoon-v1.zip)
//! - [Esolang wiki](https://esolangs.org/wiki/Talk:Spoon)

use std::iter;
use std::mem;
use std::sync::LazyLock;

use bitvec::prelude::*;

use crate::bf;
use crate::syntax::{PrefixError, PrefixTable, Tokens, VariantIndex};

/// Spoon tokens.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Token {
    /// `0`
    A,
    /// `1`
    B,
}

macro_rules! tokens[
    (@token 0) => { Token::A };
    (@token 1) => { Token::B };
    ($($token:tt)*) => { &[$(tokens!(@token $token)),+] };
];

/// Spoon instructions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Inst {
    /// Spoon instructions that are isomorphic to Brainfuck instructions.
    ///
    /// - `1` — `+`
    /// - `000` — `-`
    /// - `010` — `>`
    /// - `011` — `<`
    /// - `0011` — `]`
    /// - `00100` — `[`
    /// - `001010` — `.`
    /// - `0010110` — `,`
    Bf(bf::Inst),
    /// Outputs the stack.
    ///
    /// - `00101110` (`DEBUG`) — `#`
    Debug,
    /// Quits the program.
    ///
    /// - `00101111` (`EXIT`)
    Exit,
}

/// Prefix table for parsing Spoon instructions.
pub static TABLE: LazyLock<PrefixTable<Token, Inst>> = LazyLock::new(|| {
    let mut table = PrefixTable::new(11);
    table.insert_all();
    table
});

impl Token {
    pub fn iter_bits<T: BitStore, O: BitOrder>(
        bits: &BitSlice<T, O>,
        swap: bool,
    ) -> impl Iterator<Item = Token> + '_ {
        bits.iter()
            .by_vals()
            .map(move |bit| if bit ^ swap { Token::B } else { Token::A })
    }
}

impl Inst {
    pub fn parse<I: Iterator<Item = Token>>(
        lex: I,
    ) -> impl Iterator<Item = Result<Inst, PrefixError<Token, Inst>>> {
        let mut lex = lex.map(|tok| Ok(tok));
        let table = &*TABLE;
        iter::from_fn(move || table.parse(&mut lex))
    }
}

impl Tokens for Inst {
    type Token = Token;

    #[inline]
    fn tokens(&self) -> &'static [Token] {
        use bf::Inst::*;
        match self {
            Inst::Bf(Inc) => tokens![1],
            Inst::Bf(Dec) => tokens![0 0 0],
            Inst::Bf(Right) => tokens![0 1 0],
            Inst::Bf(Left) => tokens![0 1 1],
            Inst::Bf(Tail) => tokens![0 0 1 1],
            Inst::Bf(Head) => tokens![0 0 1 0 0],
            Inst::Bf(Output) => tokens![0 0 1 0 1 0],
            Inst::Bf(Input) => tokens![0 0 1 0 1 1 0],
            Inst::Debug => tokens![0 0 1 0 1 1 1 0],
            Inst::Exit => tokens![0 0 1 0 1 1 1 1],
        }
    }
}

impl From<bf::Inst> for Inst {
    fn from(inst: bf::Inst) -> Self {
        Inst::Bf(inst)
    }
}

impl VariantIndex for Token {
    const COUNT: u32 = 2;
    #[inline]
    fn variant(index: u32) -> Self {
        unsafe { mem::transmute(index as u8) }
    }
    #[inline]
    fn index(&self) -> u32 {
        *self as u32
    }
}

impl VariantIndex for Inst {
    const COUNT: u32 = 10;
    #[inline]
    fn variant(index: u32) -> Self {
        unsafe { mem::transmute(index as u8) }
    }
    #[inline]
    fn index(&self) -> u32 {
        unsafe { mem::transmute::<_, u8>(*self) as u32 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! insts[
        (@inst +) => { Inst::Bf(bf::Inst::Inc) };
        (@inst -) => { Inst::Bf(bf::Inst::Dec) };
        (@inst >) => { Inst::Bf(bf::Inst::Right) };
        (@inst <) => { Inst::Bf(bf::Inst::Left) };
        (@inst ']') => { Inst::Bf(bf::Inst::Tail) };
        (@inst '[') => { Inst::Bf(bf::Inst::Head) };
        (@inst .) => { Inst::Bf(bf::Inst::Output) };
        (@inst ,) => { Inst::Bf(bf::Inst::Input) };
        (@inst #) => { Inst::Debug };
        (@inst DEBUG) => { Inst::Debug };
        (@inst EXIT) => { Inst::Exit };
        ($($inst:tt)*) => { &[$(insts!(@inst $inst)),+] };
    ];

    #[test]
    fn parse() {
        // From `spoon-v1.zip/Examples/HelloWorld.sp`
        let bits = bitvec![
            0, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0,
            1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 1, 1, 0, 0, 1, 0, 1, 0, 0, 1, 0, 1, 1, 1, 1, 1, 1, 1, 0,
            0, 1, 0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 1, 1, 1, 0, 0, 1, 0,
            1, 0, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 0, 1, 0, 0, 0, 1, 0, 1, 0, 1, 1, 1, 0, 0, 1, 0, 1,
            0, 0, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 1, 1, 0, 0, 0, 0,
            0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 0, 0, 1, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 1, 1,
            0, 0, 1, 0, 1, 0, 0, 1, 0, 1, 1, 1, 1, 1, 1, 0, 0, 1, 0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 1,
            0, 0, 0, 0, 0, 0, 1, 1, 0, 1, 1, 0, 0, 1, 0, 1, 0, 1, 1, 1, 0, 0, 1, 0, 1, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 1, 0, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0,
        ];
        let insts = Inst::parse(Token::iter_bits(&bits, false))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        const INSTS: &'static [Inst] = insts![
            > + + + + + + + + + '[' < + + + + + + + + > - ']' < . > + + + + + + + '[' < + + + + > -
            ']' < + . + + + + + + + . . + + + . > + + + + + + + + + + + '[' < - - - - - - > - ']' <
            - . - - - - - - - - - - - - . > + + + + + + + + + + + '[' < + + + + + > - ']' < . > + +
            + + + + '[' < + + + + > - ']' < . + + + . - - - - - - . - - - - - - - - . > + + + + + +
            + + + + + '[' < - - - - - - > - ']' < - .
        ];
        assert_eq!(INSTS, insts);
    }
}
