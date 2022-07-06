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
use rug::Integer;
use strum::IntoEnumIterator;

use crate::ws::inst::{Features, Inst, Int, Label, Opcode};
use crate::ws::lex::{LexError, Lexer};
use crate::ws::token::{token_vec, Token::*, TokenSeq};

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
        let table = ParseTable::with_features(features)?;
        Ok(Parser { table, lex })
    }

    #[inline]
    fn parse_arg(&mut self, opcode: Opcode) -> Inst {
        opcode.parse_arg(self)
    }

    pub(crate) fn parse_int(&mut self, opcode: Opcode) -> Result<Int, ParseError> {
        let bits = self.parse_bitvec(opcode)?;
        // TODO: Convert int
        Ok(Int { bits, int: Integer::new() })
    }

    pub(crate) fn parse_label(&mut self, opcode: Opcode) -> Result<Label, ParseError> {
        let bits = self.parse_bitvec(opcode)?;
        // TODO: Convert num and name
        Ok(Label { bits, num: None, name: None })
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

    pub fn with_features(features: Features) -> Result<Self, ParserError> {
        let mut table = ParseTable::new();
        for opcode in Opcode::iter() {
            if opcode.feature().map_or(true, |f| features.contains(f)) {
                table.register(opcode)?;
            }
        }
        Ok(table)
    }

    #[inline]
    pub fn parser<L: Lexer>(self, lex: L) -> Parser<L> {
        Parser { table: self, lex }
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
