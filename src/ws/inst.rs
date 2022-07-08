// Copyright (C) 2022 Andrew Archibald
//
// yspace2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with yspace2. If not, see http://www.gnu.org/licenses/.

use std::fmt::{self, Display, Formatter};

use bitvec::vec::BitVec;
use enumset::{EnumSet, EnumSetType};
use paste::paste;
use strum::{Display, EnumIter, IntoStaticStr};

use crate::ws::lex::Lexer;
use crate::ws::parse::ParseError;
use crate::ws::token::{token_vec, Token::*, TokenVec};

pub type RawInst = Inst<BitVec, BitVec>;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum InstError {
    ParseError(ParseError),
}

pub type Features = EnumSet<Feature>;

#[derive(Debug, Hash)]
#[derive(Display, EnumSetType)]
#[strum(serialize_all = "snake_case")]
pub enum Feature {
    #[strum(serialize = "wspace 0.3")]
    Wspace0_3,
    Shuffle,
    #[strum(serialize = "dump_stack/dump_heap")]
    DumpStackHeap,
    DumpTrace,
}

macro_rules! map(
    ( , $then:tt) => {};
    ($optional:tt, $then:tt) => { $then };
);

macro_rules! map_or(
    ( , $($then:expr)?, $else:expr) => { $else };
    ($optional:expr, $($then:expr)?, $else:expr) => { $($then)? };
);

macro_rules! insts {
    ($([$($seq:expr)+ $(; $arg:ident)?] $(if $feature:ident)? => $opcode:ident),+$(,)?) => {
        #[derive(Clone, Debug, PartialEq, Eq, Hash)]
        pub enum Inst<Int, Label> {
            $($opcode $(($arg))?),+,
            Error(InstError),
        }

        impl RawInst {
            #[inline]
            pub const fn opcode(&self) -> Opcode {
                match self {
                    $(Inst::$opcode $((map!($arg, _)))? => Opcode::$opcode),+,
                    Inst::Error(_) => panic!("no opcode for Error"),
                }
            }
        }

        impl Display for RawInst {
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

        impl const From<Opcode> for Inst<(), ()> {
            #[inline]
            fn from(opcode: Opcode) -> Self {
                match opcode {
                    $(Opcode::$opcode => Inst::$opcode $((map!($arg, ())))?),+,
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum InstArg<I, L> {
    Int(I),
    Label(L),
}

impl<I1, L1> Inst<I1, L1> {
    pub fn map_arg<I2, L2, E, F>(self, f: F) -> Inst<I2, L2>
    where
        E: Into<InstError>,
        F: FnOnce(Opcode, InstArg<I1, L1>) -> Result<InstArg<I2, L2>, E>,
    {
        macro_rules! map_int(($opcode:ident, $n:ident) => {
            match f(Opcode::$opcode, InstArg::Int($n)) {
                Ok(InstArg::Int(n)) => $opcode(n),
                Ok(InstArg::Label(_)) => unreachable!(),
                Err(err) => Inst::from(err),
            }
        });
        macro_rules! map_label(($opcode:ident, $l:ident) => {
            match f(Opcode::$opcode, InstArg::Label($l)) {
                Ok(InstArg::Int(_)) => unreachable!(),
                Ok(InstArg::Label(l)) => $opcode(l),
                Err(err) => Inst::from(err),
            }
        });
        use Inst::*;
        match self {
            Push(n) => map_int!(Push, n),
            Dup => Dup,
            Copy(n) => map_int!(Copy, n),
            Swap => Swap,
            Drop => Drop,
            Slide(n) => map_int!(Slide, n),
            Add => Add,
            Sub => Sub,
            Mul => Mul,
            Div => Div,
            Mod => Mod,
            Store => Store,
            Retrieve => Retrieve,
            Label(l) => map_label!(Label, l),
            Call(l) => map_label!(Call, l),
            Jmp(l) => map_label!(Jmp, l),
            Jz(l) => map_label!(Jz, l),
            Jn(l) => map_label!(Jn, l),
            Ret => Ret,
            End => End,
            Printc => Printc,
            Printi => Printi,
            Readc => Readc,
            Readi => Readi,
            Shuffle => Shuffle,
            DumpStack => DumpStack,
            DumpHeap => DumpHeap,
            DumpTrace => DumpTrace,
            Error(err) => Error(err),
        }
    }
}

impl<I, L, E: Into<InstError>> const From<E> for Inst<I, L> {
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
    [L S S; Label] => Label,
    [L S T; Label] => Call,
    [L S L; Label] => Jmp,
    [L T S; Label] => Jz,
    [L T T; Label] => Jn,
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
