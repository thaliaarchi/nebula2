// Copyright (C) 2022 Andrew Archibald
//
// yspace2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with yspace2. If not, see http://www.gnu.org/licenses/.

use std::collections::HashMap;
use std::fmt::{self, Debug, Formatter};
use std::iter::FusedIterator;

use bitvec::prelude::BitVec;
use strum::IntoEnumIterator;

use crate::ws::inst::{Features, Inst, InstArg, Opcode, RawInst};
use crate::ws::lex::{LexError, Lexer};
use crate::ws::token::{token_vec, Token::*, TokenSeq, TokenVec};

#[derive(Clone, Debug)]
pub struct Parser<L: Lexer> {
    table: ParseTable,
    lex: L,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ParseError {
    LexError(LexError, TokenVec),
    UnknownOpcode(TokenVec),
    IncompleteInst(TokenVec, Vec<Opcode>),
    UnterminatedArg(Opcode),
}

impl<L: Lexer> Parser<L> {
    pub fn new(lex: L, features: Features) -> Result<Self, ParserError> {
        let table = ParseTable::with_features(features)?;
        Ok(Parser { table, lex })
    }

    fn parse_arg(&mut self, opcode: Opcode) -> RawInst {
        Inst::from(opcode).map_arg(|opcode, arg| {
            let mut bits = BitVec::new();
            loop {
                match self.lex.next() {
                    Some(Ok(S)) => bits.push(false),
                    Some(Ok(T)) => bits.push(true),
                    Some(Ok(L)) => break,
                    Some(Err(err)) => return Err(ParseError::LexError(err, opcode.tokens())),
                    None => return Err(ParseError::UnterminatedArg(opcode)),
                }
            }
            match arg {
                InstArg::Int(()) => Ok(InstArg::Int(bits)),
                InstArg::Label(()) => Ok(InstArg::Label(bits)),
            }
        })
    }
}

impl<L: Lexer> Iterator for Parser<L> {
    type Item = RawInst;

    fn next(&mut self) -> Option<Self::Item> {
        use ParseError::*;
        let mut seq = TokenSeq::new();
        let mut prefix = &Vec::new();
        loop {
            match self.lex.next() {
                Some(Ok(tok)) => seq.push(tok),
                Some(Err(err)) => return Some(Inst::from(LexError(err, seq.into()))),
                None if seq.is_empty() => return None,
                None => return Some(Inst::from(IncompleteInst(seq.into(), prefix.clone()))),
            }
            match self.table.get(seq) {
                ParseEntry::Unknown => return Some(Inst::from(UnknownOpcode(seq.into()))),
                ParseEntry::Prefix(opcodes) => prefix = opcodes,
                ParseEntry::Terminal(opcode) => return Some(self.parse_arg(*opcode)),
            }
        }
    }
}

impl<L: Lexer + FusedIterator> const FusedIterator for Parser<L> {}

#[derive(Clone)]
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
    Conflict {
        prefix: TokenVec,
        opcodes: Vec<Opcode>,
    },
    NoTokens(Opcode),
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
    fn get(&self, seq: TokenSeq) -> &ParseEntry {
        if seq <= Self::DENSE_MAX {
            &self.dense[seq.as_usize()]
        } else {
            &self.sparse[&seq]
        }
    }

    #[inline]
    fn get_mut(&mut self, seq: TokenSeq) -> &mut ParseEntry {
        if seq <= Self::DENSE_MAX {
            &mut self.dense[seq.as_usize()]
        } else {
            self.sparse.entry(seq).or_default()
        }
    }

    pub fn register(&mut self, opcode: Opcode) -> Result<(), ParserError> {
        #[inline]
        const fn conflict(seq: TokenSeq, opcodes: Vec<Opcode>) -> Result<(), ParserError> {
            Err(ParserError::Conflict { prefix: seq.into(), opcodes })
        }
        use ParseEntry::*;
        let toks = opcode.tokens();
        if toks.len() == 0 {
            return Err(ParserError::NoTokens(opcode));
        }
        let mut seq = TokenSeq::new();
        for tok in toks {
            let entry = self.get_mut(seq);
            match entry {
                Unknown => *entry = Prefix(vec![opcode]),
                Prefix(opcodes) => opcodes.push(opcode),
                Terminal(terminal) => return conflict(seq, vec![*terminal, opcode]),
            }
            seq.push(tok);
        }
        let entry = self.get_mut(seq);
        match entry {
            Unknown => *entry = Terminal(opcode),
            Prefix(opcodes) => {
                let mut opcodes = opcodes.clone();
                opcodes.push(opcode);
                return conflict(seq, opcodes);
            }
            Terminal(terminal) => return conflict(seq, vec![*terminal, opcode]),
        }
        Ok(())
    }
}

impl Debug for ParseTable {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        struct EntryDebug<'a>(TokenSeq, &'a ParseEntry);
        impl<'a> Debug for EntryDebug<'a> {
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                write!(f, "{}{:?}: {:?}", self.0 .0, TokenVec::from(self.0), self.1)
            }
        }

        let dense = self
            .dense
            .iter()
            .enumerate()
            .map(|(i, e)| EntryDebug(TokenSeq(i as u16), e))
            .collect::<Vec<_>>();
        let mut sparse = self
            .sparse
            .iter()
            .map(|(&seq, e)| EntryDebug(seq, e))
            .collect::<Vec<_>>();
        sparse.sort_by(|a, b| a.0.cmp(&b.0));

        f.debug_struct("ParseTable")
            .field("dense", &dense)
            .field("sparse", &sparse)
            .finish()
    }
}
