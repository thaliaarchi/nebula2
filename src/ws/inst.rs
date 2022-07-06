// Copyright (C) 2022 Andrew Archibald
//
// yspace2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with yspace2. If not, see http://www.gnu.org/licenses/.

use std::fmt::{self, Display, Formatter};

use bitvec::prelude::BitVec;
use enumset::{EnumSet, EnumSetType};
use paste::paste;
use strum::{Display, EnumIter, IntoStaticStr};

use crate::ws::lex::Lexer;
use crate::ws::parse::{ParseError, Parser};
use crate::ws::token::Token::*;
use crate::ws::token_vec::{token_vec, TokenVec};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Int {
    pub sign: Sign,
    pub bits: BitVec,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Sign {
    Pos,
    Neg,
    Empty,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Uint {
    pub bits: BitVec,
}

pub type Features = EnumSet<Feature>;

#[derive(Debug, Hash)]
#[derive(EnumSetType)]
pub enum Feature {
    Wspace0_3,
    Shuffle,
    DumpStackHeap,
    DumpTrace,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum InstError {
    ParseError(ParseError),
}

macro_rules! map(
    ( , $then:tt) => { };
    ($optional:tt, $then:tt) => { $then };
);

macro_rules! map_or(
    ( , $($then:expr)?, $else:expr) => { $else };
    ($optional:expr, $($then:expr)?, $else:expr) => { $($then)? };
);

macro_rules! insts {
    ($([$($seq:expr)+ $(; $arg:ident)?] $(if $feature:ident)? => $opcode:ident),+$(,)?) => {
        #[derive(Clone, Debug, PartialEq, Eq, Hash)]
        pub enum Inst {
            $($opcode $(($arg))?),+,
            Error(InstError),
        }

        impl Inst {
            #[inline]
            pub const fn opcode(&self) -> Opcode {
                match self {
                    $(Inst::$opcode $((map!($arg, _)))? => Opcode::$opcode),+,
                    Inst::Error(_) => panic!("no opcode for Error"),
                }
            }
        }

        impl Display for Inst {
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                // TODO: uses Debug for arguments
                f.write_str(self.opcode().into())?;
                paste! {
                    match self {
                        $(Inst::$opcode $(([<$arg:snake>]))? => {
                            map_or!($($arg)?, write!(f, " {:?}", $([<$arg:snake>])?), Ok(()))
                        }),+,
                        Inst::Error(err) => write!(f, " {:?}", err),
                    }
                }
            }
        }

        #[repr(u8)]
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[derive(Display, EnumIter, IntoStaticStr)]
        #[strum(serialize_all = "snake_case")]
        pub enum Opcode {
            $($opcode),+,
        }

        impl Opcode {
            pub(crate) fn parse_arg<L: Lexer>(&self, parser: &mut Parser<L>) -> Inst {
                paste! {
                    match self {
                        $(Opcode::$opcode => map_or!($($arg)?,
                            $(parser.[<parse_ $arg:snake>](Opcode::$opcode).map_or_else(Inst::from, Inst::$opcode))?,
                            Inst::$opcode
                        )),+,
                    }
                }
            }

            #[inline]
            pub const fn tokens(&self) -> TokenVec {
                match self {
                    $(Opcode::$opcode => const { token_vec![$($seq)+] }),+
                }
            }

            #[inline]
            pub const fn feature(&self) -> Option<Feature> {
                match self {
                    $(Opcode::$opcode => {
                        map_or!($($feature)?, $(Some(Feature::$feature))?, None)
                    }),+,
                }
            }
        }
    }
}

impl<E: Into<InstError>> const From<E> for Inst {
    #[inline]
    fn from(err: E) -> Self {
        Inst::Error(err.into())
    }
}

impl const From<ParseError> for InstError {
    #[inline]
    fn from(err: ParseError) -> Self {
        InstError::ParseError(err)
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
