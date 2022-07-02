// Copyright (C) 2022 Andrew Archibald
//
// yspace2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with yspace2. If not, see http://www.gnu.org/licenses/.

use crate::ws::token::Token::{self, *};

macro_rules! get_max_id(
    ([$($seq:expr)+ $(; $arg:ty)?] => $variant:ty, $($tail:tt)*) => {
        const {
            let mut max_id = $crate::ws::inst::Inst::id(&[$($seq),+]);
            get_max_id!(max_id = $($tail)*);
            max_id
        }
    };
    ($max_id:ident = [$($seq:expr)+ $(; $arg:ty)?] => $variant:ty, $($tail:tt)*) => {
        let id = $crate::ws::inst::Inst::id(&[$($seq),+]);
        if id > $max_id {
            $max_id = id;
        }
        get_max_id!($max_id = $($tail)*);
    };
    ($max_id:ident =) => {}
);

macro_rules! register_inst(
    ($map:ident = [$($seq:expr)+; $arg:ty] => $variant:ty, $($tail:tt)*) => {
        $map[$crate::ws::inst::Inst::id(&[$($seq),+])] = concat!(stringify!($variant), "(", stringify!($arg), "::parse_ws())");
        register_inst!($map = $($tail)*);
    };
    ($map:ident = [$($seq:expr)+] => $variant:ty, $($tail:tt)*) => {
        $map[$crate::ws::inst::Inst::id(&[$($seq),+])] = stringify!($variant);
        register_inst!($map = $($tail)*);
    };
    ($map:ident =) => {}
);

macro_rules! parser {
    ($($insts:tt)+) => {
        {
            const MAX_ID: usize = get_max_id!($($insts)+);
            let mut insts: [&'static str; MAX_ID] = [""; MAX_ID];
            register_inst!(insts = $($insts)+);
        }
    };
}

pub struct Int {}
pub struct Uint {}

pub enum Inst {
    Push(Int),
    Dup,
    Copy(Int),
    Swap,
    Drop,
    Slide(Int),
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Store,
    Retrieve,
    Label(Uint),
    Call(Uint),
    Jmp(Uint),
    Jz(Uint),
    Jn(Uint),
    Ret,
    End,
    Printc,
    Printi,
    Readc,
    Readi,

    Shuffle,
    DumpStack,
    DumpHeap,
    DumpTrace,
}

pub fn parse() {
    parser! {
        [S S; Int]    => Push,
        [S L S]       => Dup,
        [S T S; Int]  => Copy,
        [S L T]       => Swap,
        [S L L]       => Drop,
        [S T L; Int]  => Slide,
        [T S S S]     => Add,
        [T S S T]     => Sub,
        [T S S L]     => Mul,
        [T S T S]     => Div,
        [T S T T]     => Mod,
        [T T S]       => Store,
        [T T T]       => Retrieve,
        [L S S; Uint] => Label,
        [L S T; Uint] => Call,
        [L S L; Uint] => Jmp,
        [L T S; Uint] => Jz,
        [L T T; Uint] => Jn,
        [L T L]       => Ret,
        [L L L]       => End,
        [T L S S]     => Printc,
        [T L S T]     => Printi,
        [T L T S]     => Readc,
        [T L T T]     => Readi,
        [S T T S]     => Shuffle,
        [L L S S S]   => DumpStack,
        [L L S S T]   => DumpHeap,
        [L L T]       => DumpTrace,
    }
}

impl Inst {
    #[inline]
    pub const fn id(seq: &[Token]) -> usize {
        if seq.len() == 0 {
            panic!("empty seq");
        }
        let mut id: usize = 1;
        let mut i: usize = 0;
        while i < seq.len() {
            id *= 3;
            id += match seq[i] {
                S => 0,
                T => 1,
                L => 2,
            };
            i += 1;
        }
        id - 3
    }
}
