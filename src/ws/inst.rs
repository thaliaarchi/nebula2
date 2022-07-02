// Copyright (C) 2022 Andrew Archibald
//
// yspace2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with yspace2. If not, see http://www.gnu.org/licenses/.

use enumset::{EnumSet, EnumSetType};

use self::Inst::*;
use crate::ws::parse::Parser;
use crate::ws::token::Token::{self, *};

pub struct Int {}
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

#[derive(EnumSetType)]
pub enum Feature {
    Wspace0_3,
    Shuffle,
    DumpStackHeap,
    DumpTrace,
}

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

macro_rules! parser(
    ($features:expr, { $([$($seq:expr)+ $(; $arg:ident)?] $(if $feature:ident)? => $inst:ident),+$(,)? }) => {
        {
            let features = $features;
            let mut parser = $crate::ws::parse::Parser::new();
            $(if true $(&& features.contains($crate::ws::inst::Feature::$feature))? {
                parser.register(
                    const { $crate::ws::inst::Inst::id(&[$($seq),+]) },
                    Box::new(|_p| Some($inst $(($arg::parse(_p)?))?)),
                );
            })+
            parser
        }
    }
);

impl Parser {
    pub fn with_features(features: Features) -> Parser {
        parser!(features, {
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
        })
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
