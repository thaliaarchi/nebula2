// Copyright (C) 2022 Andrew Archibald
//
// Nebula 2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with Nebula 2. If not, see http://www.gnu.org/licenses/.

use std::collections::HashMap;
use std::fmt::{self, Debug, Formatter};
use std::iter::FusedIterator;

use bitvec::vec::BitVec;
use smallvec::{smallvec, SmallVec};
use strum::IntoEnumIterator;

use crate::ws::inst::{Features, Inst, InstArg, Opcode, RawInst};
use crate::ws::lex::{LexError, Lexer};
use crate::ws::token::{token_vec, Token::*, TokenSeq, TokenVec};

#[derive(Clone, Debug)]
pub struct Parser<'a, L: Lexer> {
    table: &'a ParseTable,
    lex: L,
    partial: Option<PartialState>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ParseError {
    LexError(LexError, TokenVec),
    UnknownOpcode(TokenVec),
    IncompleteInst(TokenVec, OpcodeVec),
    UnterminatedArg(Opcode),
}

#[derive(Clone, Debug)]
enum PartialState {
    ParsingOpcode(TokenSeq),
    ParsingArg(Opcode, BitVec),
}

impl<'a, L: Lexer> Parser<'a, L> {
    pub fn new(table: &'a ParseTable, lex: L) -> Self {
        Parser { table, lex, partial: None }
    }

    fn parse_arg(&mut self, opcode: Opcode, partial: Option<BitVec>) -> RawInst {
        Inst::from(opcode).map_arg(|opcode, arg| {
            let mut bits = partial.unwrap_or_else(|| BitVec::with_capacity(64));
            loop {
                match self.lex.next() {
                    Some(Ok(S)) => bits.push(false),
                    Some(Ok(T)) => bits.push(true),
                    Some(Ok(L)) => break,
                    Some(Err(err)) => {
                        let mut tokens = opcode.tokens();
                        tokens.append_bits(&bits);
                        self.partial = Some(PartialState::ParsingArg(opcode, bits));
                        return Err(ParseError::LexError(err, tokens));
                    }
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

impl<'a, L: Lexer> Iterator for Parser<'a, L> {
    type Item = RawInst;

    fn next(&mut self) -> Option<Self::Item> {
        use {ParseEntry::*, ParseError::*};
        // Restore state, if an instruction was interrupted with a lex error
        // after being partially parsed.
        let mut seq = match self.partial.take() {
            Some(PartialState::ParsingOpcode(partial)) => partial,
            Some(PartialState::ParsingArg(opcode, bits)) => {
                return Some(self.parse_arg(opcode, Some(bits)));
            }
            None => TokenSeq::new(),
        };

        loop {
            match self.lex.next() {
                Some(Ok(tok)) => seq.push(tok),
                Some(Err(err)) => {
                    self.partial = Some(PartialState::ParsingOpcode(seq));
                    return Some(Inst::from(LexError(err, seq.into())));
                }
                None if seq.is_empty() => return None,
                None => {
                    let prefix = match self.table.get(seq) {
                        Some(Prefix(opcodes)) => opcodes.clone(),
                        _ => unreachable!(),
                    };
                    return Some(Inst::from(IncompleteInst(seq.into(), prefix)));
                }
            }
            match self.table.get(seq) {
                Some(Terminal(opcode)) => return Some(self.parse_arg(*opcode, None)),
                Some(Prefix(_)) => {}
                None => return Some(Inst::from(UnknownOpcode(seq.into()))),
            }
        }
    }
}

impl<'a, L: Lexer + FusedIterator> const FusedIterator for Parser<'a, L> {}

#[derive(Clone)]
pub struct ParseTable {
    dense: Box<[Option<ParseEntry>; Self::DENSE_LEN]>,
    sparse: HashMap<TokenSeq, Option<ParseEntry>>,
}

#[derive(Clone, Debug)]
pub enum ParseEntry {
    Terminal(Opcode),
    Prefix(OpcodeVec),
}

type OpcodeVec = SmallVec<[Opcode; 16]>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParserError {
    Conflict {
        prefix: TokenVec,
        opcodes: OpcodeVec,
    },
    NoTokens(Opcode),
}

impl ParseTable {
    const DENSE_MAX: TokenSeq = token_vec![L L L].into();
    const DENSE_LEN: usize = Self::DENSE_MAX.as_usize() + 1;

    pub fn empty() -> Self {
        ParseTable {
            dense: vec![None; Self::DENSE_LEN]
                .into_boxed_slice()
                .try_into()
                .unwrap(),
            sparse: HashMap::new(),
        }
    }

    pub fn new(features: Features) -> Self {
        let mut table = ParseTable::empty();
        for opcode in Opcode::iter() {
            if opcode.feature().map_or(true, |f| features.contains(f)) {
                table.register(opcode).unwrap();
            }
        }
        table
    }

    #[inline]
    pub fn with_all() -> Self {
        Self::new(Features::all())
    }

    #[inline]
    fn get(&self, seq: TokenSeq) -> Option<&ParseEntry> {
        if seq <= Self::DENSE_MAX {
            self.dense[seq.as_usize()].as_ref()
        } else {
            self.sparse.get(&seq).map(Option::as_ref).flatten()
        }
    }

    #[inline]
    fn get_mut(&mut self, seq: TokenSeq) -> &mut Option<ParseEntry> {
        if seq <= Self::DENSE_MAX {
            &mut self.dense[seq.as_usize()]
        } else {
            self.sparse.entry(seq).or_default()
        }
    }

    pub fn register(&mut self, opcode: Opcode) -> Result<(), ParserError> {
        #[inline]
        const fn conflict(seq: TokenSeq, opcodes: OpcodeVec) -> Result<(), ParserError> {
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
                Some(Terminal(terminal)) => return conflict(seq, smallvec![*terminal, opcode]),
                Some(Prefix(opcodes)) => opcodes.push(opcode),
                None => *entry = Some(Prefix(smallvec![opcode])),
            }
            seq.push(tok);
        }
        let entry = self.get_mut(seq);
        match entry {
            Some(Terminal(terminal)) => return conflict(seq, smallvec![*terminal, opcode]),
            Some(Prefix(opcodes)) => {
                let mut opcodes = opcodes.clone();
                opcodes.push(opcode);
                return conflict(seq, opcodes);
            }
            None => *entry = Some(Terminal(opcode)),
        }
        Ok(())
    }
}

impl Debug for ParseTable {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        struct EntryDebug<'a>(TokenSeq, Option<&'a ParseEntry>);
        impl<'a> Debug for EntryDebug<'a> {
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                write!(f, "{}{:?}: {:?}", self.0 .0, TokenVec::from(self.0), self.1)
            }
        }

        let dense = self
            .dense
            .iter()
            .enumerate()
            .map(|(i, e)| EntryDebug(TokenSeq::from(i), e.as_ref()))
            .collect::<Vec<_>>();
        let mut sparse = self
            .sparse
            .iter()
            .map(|(&seq, e)| EntryDebug(seq, e.as_ref()))
            .collect::<Vec<_>>();
        sparse.sort_by(|a, b| a.0.cmp(&b.0));

        f.debug_struct("ParseTable")
            .field("dense", &dense)
            .field("sparse", &sparse)
            .field("sparse.len", &sparse.len())
            .field("sparse.capacity", &sparse.capacity())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use static_assertions::{assert_eq_size, const_assert};

    use super::*;

    #[allow(dead_code)]
    enum ParseEntryOf<T> {
        Unknown,
        Prefix(T),
        Terminal(Opcode),
    }

    assert_eq_size!(
        ParseEntry,
        ParseEntryOf<Vec<Opcode>>,
        ParseEntryOf<SmallVec<[Opcode; 16]>>,
    );
    assert_eq_size!(OpcodeVec, Vec<Opcode>, SmallVec<[Opcode; 16]>);
    const_assert!(size_of::<SmallVec<[Opcode; 16]>>() < size_of::<SmallVec<[Opcode; 17]>>());
}
