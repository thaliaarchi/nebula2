// Copyright (C) 2022 Andrew Archibald
//
// yspace2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with yspace2. If not, see http://www.gnu.org/licenses/.

use bitvec::prelude::BitVec;
use enumset::{EnumSet, EnumSetType};
use paste::paste;
use std::fmt::{self, Display, Formatter};
use std::iter::FusedIterator;
use std::mem::transmute;

use crate::ws::parse::{ParseError, Parser};
use crate::ws::token::{Token::*, TokenSeq};
use crate::ws::token_vec::{token_vec, TokenVec};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Int {
    pub sign: Sign,
    pub bits: BitVec,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sign {
    Pos,
    Neg,
    Empty,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Uint {
    pub bits: BitVec,
}

pub type Features = EnumSet<Feature>;

#[derive(Debug, EnumSetType)]
pub enum Feature {
    Wspace0_3,
    Shuffle,
    DumpStackHeap,
    DumpTrace,
}

macro_rules! subst(
    ($optional:expr, $($then:expr)?, $else:expr) => { $($then)? };
    ($optional:expr, $($then:expr)?) => { $($then)? };
    ( , $($then:expr)?, $else:expr) => { $else };
    ( , $($then:expr)?) => { };
);

macro_rules! insts {
    ($([$($seq:expr)+ $(; $arg:ident)?] $(if $feature:ident)? => $opcode:ident),+$(,)?) => {
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub enum Inst {
            $($opcode $(($arg))?),+
        }

        impl Inst {
            #[inline]
            pub fn opcode(&self) -> Opcode {
                paste! {
                    match self {
                        $(Inst::$opcode $(([<_ $arg:snake>]))? => Opcode::$opcode),+
                    }
                }
            }
        }

        impl Display for Inst {
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                // TODO: uses Debug for arguments
                f.write_str(self.opcode().to_str())?;
                paste! {
                    match self {
                        $(Inst::$opcode $(([<$arg:snake>]))? => {
                            subst!($($arg)?, write!(f, " {:?}", $([<$arg:snake>])?), Ok(()))
                        }),+
                    }
                }
            }
        }

        #[repr(u8)]
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum Opcode {
            $($opcode),+
        }

        impl Opcode {
            const COUNT: usize = 0 $(+ subst!($opcode, 1))+;
            const NAMES: [&'static str; Self::COUNT] = [
                $(paste!(stringify!([<$opcode:snake>]))),+
            ];
            const SEQS: [TokenSeq; Self::COUNT] = [
                $(token_vec![$($seq)+].into()),+
            ];
            const TOKENS: [TokenVec; Self::COUNT] = [
                $(token_vec![$($seq)+]),+
            ];

            #[inline]
            pub fn parse_arg(&self, parser: &mut Parser) -> Result<Inst, ParseError> {
                paste! {
                    match self {
                        $(Opcode::$opcode => {
                            Ok(Inst::$opcode $((parser.[<parse_ $arg:snake>](Opcode::$opcode)?))?)
                        }),+
                    }
                }
            }

            #[inline]
            pub fn feature(&self) -> Option<Feature> {
                match self {
                    $(Opcode::$opcode => {
                        subst!($($feature)?, $(Some(Feature::$feature))?, None)
                    }),+
                }
            }
        }
    }
}

impl Display for Opcode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.to_str())
    }
}

impl Opcode {
    #[inline]
    pub fn to_str(&self) -> &'static str {
        Opcode::NAMES[*self as usize]
    }

    #[inline]
    pub fn seq(&self) -> TokenSeq {
        Opcode::SEQS[*self as usize]
    }

    #[inline]
    pub fn tokens(&self) -> TokenVec {
        Opcode::TOKENS[*self as usize]
    }

    #[inline]
    pub fn iter() -> impl Iterator<Item = Opcode> + FusedIterator {
        OpcodeIterator(0)
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
struct OpcodeIterator(u8);

impl Iterator for OpcodeIterator {
    type Item = Opcode;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.0 < Opcode::COUNT as u8 {
            let opcode = unsafe { transmute(self.0 as u8) };
            self.0 += 1;
            Some(opcode)
        } else {
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = Opcode::COUNT - self.0 as usize;
        (n, Some(n))
    }
}

impl FusedIterator for OpcodeIterator {}

insts! {
    [S S; Int] => Push,
    [S L S] => Dup,
    [S T S; Int] if Wspace0_3 => Copy,
    [S L T] => Swap,
    [S L L] => Drop,
    [S T L; Int] if Wspace0_3 => Slide,
    [T S S S] => Add,
    [T S S T] => Sub,
    [T S S L] => Mul,
    [T S T S] => Div,
    [T S T T] => Mod,
    [T T S] => Store,
    [T T T] => Retrieve,
    [L S S; Uint] => Label,
    [L S T; Uint] => Call,
    [L S L; Uint] => Jmp,
    [L T S; Uint] => Jz,
    [L T T; Uint] => Jn,
    [L T L] => Ret,
    [L L L] => End,
    [T L S S] => Printc,
    [T L S T] => Printi,
    [T L T S] => Readc,
    [T L T T] => Readi,
    [S T T S] if Shuffle => Shuffle,
    [L L S S S] if DumpStackHeap => DumpStack,
    [L L S S T] if DumpStackHeap => DumpHeap,
    [L L T] if DumpTrace => DumpTrace,
}
