// Copyright (C) 2022 Andrew Archibald
//
// yspace2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with yspace2. If not, see http://www.gnu.org/licenses/.

use enumset::{EnumSet, EnumSetType};
use paste::paste;
use std::fmt::{self, Display, Formatter};

use crate::ws::parse::{ParseTable, ParserError};
use crate::ws::token::{Token::*, TokenSeq};

#[derive(Clone, Debug)]
pub struct Int {}

#[derive(Clone, Debug)]
pub struct Uint {}

impl Int {
    pub fn parse(_p: &mut ParseTable) -> Option<Int> {
        todo!();
    }
}

impl Uint {
    pub fn parse(_p: &mut ParseTable) -> Option<Uint> {
        todo!();
    }
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
        #[derive(Clone, Debug)]
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
                $(TokenSeq::from_tokens(&[$($seq),+])),+
            ];

            #[inline]
            pub fn parse_arg(&self, parser: &mut ParseTable) -> Option<Inst> {
                match self {
                    $(Opcode::$opcode => Some(Inst::$opcode $(($arg::parse(parser)?))?)),+
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

        impl ParseTable {
            pub fn insert_insts(&mut self) -> Result<(), ParserError> {
                $(self.insert(&[$($seq),+], Opcode::$opcode)?;)+
                Ok(())
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
}

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
