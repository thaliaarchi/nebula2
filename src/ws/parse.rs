// Copyright (C) 2022 Andrew Archibald
//
// yspace2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with yspace2. If not, see http://www.gnu.org/licenses/.

use std::collections::HashMap;
use std::iter::FusedIterator;

use bitvec::prelude::BitVec;
use strum::IntoEnumIterator;

use crate::ws::inst::{Features, Inst, Int, Opcode, Sign, Uint};
use crate::ws::lex::{LexError, Lexer};
use crate::ws::token::{Token::*, TokenSeq};
use crate::ws::token_vec::token_vec;

#[derive(Clone, Debug)]
pub struct Parser<L: Lexer> {
    table: ParseTable,
    lex: L,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ParseError {
    LexError(LexError, TokenSeq),
    UnknownOpcode(TokenSeq),
    IncompleteInst(TokenSeq, Vec<Opcode>),
    UnterminatedArg(Opcode),
}

impl<L: Lexer> Parser<L> {
    pub fn new(lex: L, features: Features) -> Result<Self, ParserError> {
        let mut table = ParseTable::new();
        for opcode in Opcode::iter() {
            if opcode.feature().map_or(true, |f| features.contains(f)) {
                table.register(opcode)?;
            }
        }
        Ok(Parser { table, lex })
    }

    #[inline]
    fn parse_arg(&mut self, opcode: Opcode) -> Inst {
        match opcode {
            Opcode::Push => match self.parse_int(Opcode::Push) {
                Ok(arg) => Inst::Push(arg),
                Err(err) => Inst::from(err),
            },
            Opcode::Dup => Inst::Dup,
            Opcode::Copy => match self.parse_int(Opcode::Copy) {
                Ok(arg) => Inst::Copy(arg),
                Err(err) => Inst::from(err),
            },
            Opcode::Swap => Inst::Swap,
            Opcode::Drop => Inst::Drop,
            Opcode::Slide => match self.parse_int(Opcode::Slide) {
                Ok(arg) => Inst::Slide(arg),
                Err(err) => Inst::from(err),
            },
            Opcode::Add => Inst::Add,
            Opcode::Sub => Inst::Sub,
            Opcode::Mul => Inst::Mul,
            Opcode::Div => Inst::Div,
            Opcode::Mod => Inst::Mod,
            Opcode::Store => Inst::Store,
            Opcode::Retrieve => Inst::Retrieve,
            Opcode::Label => match self.parse_uint(Opcode::Label) {
                Ok(arg) => Inst::Label(arg),
                Err(err) => Inst::from(err),
            },
            Opcode::Call => match self.parse_uint(Opcode::Call) {
                Ok(arg) => Inst::Call(arg),
                Err(err) => Inst::from(err),
            },
            Opcode::Jmp => match self.parse_uint(Opcode::Jmp) {
                Ok(arg) => Inst::Jmp(arg),
                Err(err) => Inst::from(err),
            },
            Opcode::Jz => match self.parse_uint(Opcode::Jz) {
                Ok(arg) => Inst::Jz(arg),
                Err(err) => Inst::from(err),
            },
            Opcode::Jn => match self.parse_uint(Opcode::Jn) {
                Ok(arg) => Inst::Jn(arg),
                Err(err) => Inst::from(err),
            },
            Opcode::Ret => Inst::Ret,
            Opcode::End => Inst::End,
            Opcode::Printc => Inst::Printc,
            Opcode::Printi => Inst::Printi,
            Opcode::Readc => Inst::Readc,
            Opcode::Readi => Inst::Readi,
            Opcode::Shuffle => Inst::Shuffle,
            Opcode::DumpStack => Inst::DumpStack,
            Opcode::DumpHeap => Inst::DumpHeap,
            Opcode::DumpTrace => Inst::DumpTrace,
        }
    }

    fn parse_int(&mut self, opcode: Opcode) -> Result<Int, ParseError> {
        let sign = match self.lex.next() {
            Some(Ok(S)) => Sign::Pos,
            Some(Ok(T)) => Sign::Neg,
            Some(Ok(L)) => Sign::Empty,
            Some(Err(err)) => return Err(ParseError::LexError(err, opcode.tokens().into())),
            None => return Err(ParseError::UnterminatedArg(opcode)),
        };
        let bits = if sign == Sign::Empty {
            BitVec::new()
        } else {
            self.parse_bitvec(opcode)?
        };
        Ok(Int { sign, bits })
    }

    fn parse_uint(&mut self, opcode: Opcode) -> Result<Uint, ParseError> {
        let bits = self.parse_bitvec(opcode)?;
        Ok(Uint { bits })
    }

    fn parse_bitvec(&mut self, opcode: Opcode) -> Result<BitVec, ParseError> {
        let mut bits = BitVec::new();
        loop {
            match self.lex.next() {
                Some(Ok(S)) => bits.push(false),
                Some(Ok(T)) => bits.push(true),
                Some(Ok(L)) => return Ok(bits),
                Some(Err(err)) => return Err(ParseError::LexError(err, opcode.tokens().into())),
                None => return Err(ParseError::UnterminatedArg(opcode)),
            }
        }
    }
}

impl<L: Lexer> Iterator for Parser<L> {
    type Item = Inst;

    fn next(&mut self) -> Option<Self::Item> {
        let mut seq = TokenSeq::new();
        let mut prefix = &Vec::new();
        loop {
            match self.lex.next() {
                Some(Ok(tok)) => seq.push(tok),
                Some(Err(err)) => return Some(Inst::from(ParseError::LexError(err, seq))),
                None if seq.is_empty() => return None,
                None => return Some(Inst::from(ParseError::IncompleteInst(seq, prefix.clone()))),
            }
            match self.table.get(seq) {
                ParseEntry::Unknown => return Some(Inst::from(ParseError::UnknownOpcode(seq))),
                ParseEntry::Prefix(opcodes) => prefix = opcodes,
                ParseEntry::Terminal(opcode) => return Some(self.parse_arg(*opcode)),
            }
        }
    }
}

impl<L: Lexer + FusedIterator> const FusedIterator for Parser<L> {}

#[derive(Clone, Debug)]
pub struct ParseTable {
    dense: Box<[ParseEntry; Self::DENSE_LEN]>,
    sparse: HashMap<TokenSeq, ParseEntry>,
}

#[derive(Clone, Debug, Default)]
pub enum ParseEntry {
    #[default]
    Unknown,
    Prefix(Vec<Opcode>),
    Terminal(Opcode),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParserError {
    Conflict { seq: TokenSeq, opcodes: Vec<Opcode> },
    EmptyTokenSeq(Opcode),
}

impl ParseTable {
    const DENSE_MAX: TokenSeq = token_vec![L L L].into();
    const DENSE_LEN: usize = Self::DENSE_MAX.as_usize() + 1;

    pub fn new() -> Self {
        ParseTable {
            dense: vec![ParseEntry::Unknown; Self::DENSE_LEN]
                .into_boxed_slice()
                .try_into()
                .unwrap(),
            sparse: HashMap::new(),
        }
    }

    #[inline]
    pub fn get(&self, seq: TokenSeq) -> &ParseEntry {
        if seq <= Self::DENSE_MAX {
            &self.dense[seq.as_usize()]
        } else {
            &self.sparse[&seq]
        }
    }

    #[inline]
    pub fn get_mut(&mut self, seq: TokenSeq) -> &mut ParseEntry {
        if seq <= Self::DENSE_MAX {
            &mut self.dense[seq.as_usize()]
        } else {
            self.sparse.entry(seq).or_default()
        }
    }

    pub fn register(&mut self, opcode: Opcode) -> Result<(), ParserError> {
        let toks = opcode.tokens();
        if toks.len() == 0 {
            return Err(ParserError::EmptyTokenSeq(opcode));
        }
        let mut seq = TokenSeq::new();
        for tok in toks {
            let entry = self.get_mut(seq);
            match entry {
                ParseEntry::Unknown => *entry = ParseEntry::Prefix(vec![opcode]),
                ParseEntry::Prefix(opcodes) => opcodes.push(opcode),
                ParseEntry::Terminal(terminal) => {
                    let opcodes = vec![*terminal, opcode];
                    return Err(ParserError::Conflict { seq, opcodes });
                }
            }
            seq.push(tok);
        }
        let entry = self.get_mut(seq);
        match entry {
            ParseEntry::Unknown => *entry = ParseEntry::Terminal(opcode),
            ParseEntry::Prefix(opcodes) => {
                let mut opcodes = opcodes.clone();
                opcodes.push(opcode);
                return Err(ParserError::Conflict { seq, opcodes });
            }
            ParseEntry::Terminal(terminal) => {
                let opcodes = vec![*terminal, opcode];
                return Err(ParserError::Conflict { seq, opcodes });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ws::lex::Utf8Lexer;
    use crate::ws::tests::{tutorial_insts, TUTORIAL_STL};
    use crate::ws::token::Mapping;

    #[test]
    fn parse_tutorial() -> Result<(), ParseError> {
        let lex = Utf8Lexer::new(TUTORIAL_STL.as_bytes().to_owned(), Mapping::<char>::STL);
        let parser = Parser::new(lex, Features::all()).unwrap();
        let insts = parser.collect::<Vec<_>>();
        assert_eq!(tutorial_insts(), insts);
        Ok(())
    }
}
