// Copyright (C) 2022 Andrew Archibald
//
// yspace2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with yspace2. If not, see http://www.gnu.org/licenses/.

use enumset::{EnumSet, EnumSetType};
use paste::paste;

use crate::ws::parse::Parser;
use crate::ws::token::{Token, Token::*, TokenSeq};

#[derive(Clone, Debug)]
pub struct Int {}

#[derive(Clone, Debug)]
pub struct Uint {}

impl Int {
    pub fn parse(_p: &mut Parser) -> Option<Int> {
        todo!();
    }
}

impl Uint {
    pub fn parse(_p: &mut Parser) -> Option<Uint> {
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

macro_rules! match_optional(
    ($optional:expr, $($then:expr)?, $else:expr) => { $($then)? };
    ( , $($then:expr)?, $else:expr) => { $else };
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

        #[repr(u8)]
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum Opcode {
            $($opcode),+
        }

        impl Opcode {
            #[inline]
            pub fn parse_arg(&self, parser: &mut Parser) -> Option<Inst> {
                match self {
                    $(Opcode::$opcode => Some(Inst::$opcode $(($arg::parse(parser)?))?)),+
                }
            }

            #[inline]
            pub fn feature(&self) -> Option<Feature> {
                match self {
                    $(Opcode::$opcode =>
                        match_optional!($($feature)?, $(Some(Feature::$feature))?, None)),+
                }
            }
        }

        const MAX_SEQ: TokenSeq = {
            let seqs: &[&[Token]] = &[$(&[$($seq),+]),+];
            let mut max = TokenSeq::from_tokens(seqs[0]);
            let mut i = 1;
            while i < seqs.len() {
                let seq = TokenSeq::from_tokens(seqs[i]);
                // TODO: derived PartialOrd is not const
                if seq.0 > max.0 {
                    max = seq;
                }
                i += 1;
            }
            max
        };

        impl Parser {
            pub fn new() -> Self {
                let mut parser = Parser::with_len(MAX_SEQ.0 + 1);
                $(parser.register(&[$($seq),+], Opcode::$opcode);)+
                parser
            }
        }
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
