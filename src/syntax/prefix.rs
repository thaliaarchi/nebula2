// Copyright (C) 2022 Andrew Archibald
//
// Nebula 2 is free software: you can redistribute it and/or modify it under the
// terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option) any
// later version. You should have received a copy of the GNU Lesser General
// Public License along with Nebula 2. If not, see http://www.gnu.org/licenses/.

use std::collections::HashMap;
use std::fmt::{self, Debug, Formatter};
use std::hash::Hash;
use std::iter::FusedIterator;

use bitvec::vec::BitVec;
use smallvec::{smallvec, SmallVec};
use strum::IntoEnumIterator;

use crate::syntax::{FromRepr, TokenSeq};
use crate::text::EncodingError;
use crate::ws::inst::{Features, Inst, InstArg, Opcode, RawInst};
use crate::ws::token::{token_vec, Lexer, Token, Token::*, TokenVec};

#[derive(Clone, Debug)]
pub struct PrefixParser<'a, L: Lexer> {
    table: &'a PrefixTable<Token>,
    lex: L,
    partial: Option<PartialState>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ParseError {
    EncodingError(EncodingError, TokenVec),
    UnknownOpcode(TokenVec),
    IncompleteInst(TokenVec, OpcodeVec),
    UnterminatedArg(Opcode),
}

#[derive(Clone, Debug)]
enum PartialState {
    ParsingOpcode(TokenSeq<Token>),
    ParsingArg(Opcode, BitVec),
}

impl<'a, L: Lexer> PrefixParser<'a, L> {
    pub fn new(table: &'a PrefixTable<Token>, lex: L) -> Self {
        PrefixParser { table, lex, partial: None }
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
                        return Err(ParseError::EncodingError(err, tokens));
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

impl<'a, L: Lexer> Iterator for PrefixParser<'a, L> {
    type Item = RawInst;

    fn next(&mut self) -> Option<Self::Item> {
        use {ParseError::*, PrefixEntry::*};
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
                    return Some(Inst::from(EncodingError(err, seq.into())));
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

impl<'a, L: Lexer + FusedIterator> const FusedIterator for PrefixParser<'a, L> {}

#[derive(Clone)]
pub struct PrefixTable<T> {
    dense: Box<[Option<PrefixEntry>]>,
    sparse: HashMap<TokenSeq<T>, Option<PrefixEntry>>,
}

#[derive(Clone, Debug)]
enum PrefixEntry {
    Terminal(Opcode),
    Prefix(OpcodeVec),
}

type OpcodeVec = SmallVec<[Opcode; 16]>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TableError<T> {
    Conflict {
        prefix: TokenSeq<T>,
        opcodes: OpcodeVec,
    },
    NoTokens(Opcode),
}

impl<T> PrefixTable<T>
where
    T: Copy + Eq + FromRepr + Hash,
{
    #[inline]
    pub fn new(dense_len: usize) -> Self {
        PrefixTable {
            dense: vec![None; dense_len].into_boxed_slice(),
            sparse: HashMap::new(),
        }
    }

    #[inline]
    fn get(&self, seq: TokenSeq<T>) -> Option<&PrefixEntry> {
        if seq.as_usize() < self.dense.len() {
            self.dense[seq.as_usize()].as_ref()
        } else {
            self.sparse.get(&seq).map(Option::as_ref).flatten()
        }
    }

    #[inline]
    fn get_mut(&mut self, seq: TokenSeq<T>) -> &mut Option<PrefixEntry> {
        if seq.as_usize() < self.dense.len() {
            &mut self.dense[seq.as_usize()]
        } else {
            self.sparse.entry(seq).or_default()
        }
    }

    pub fn register(&mut self, toks: &[T], opcode: Opcode) -> Result<(), TableError<T>> {
        let conflict =
            |seq: TokenSeq<T>, opcodes| Err(TableError::Conflict { prefix: seq, opcodes });
        use PrefixEntry::*;
        if toks.len() == 0 {
            return Err(TableError::NoTokens(opcode));
        }
        let mut seq = TokenSeq::new();
        for &tok in toks {
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

impl PrefixTable<Token> {
    pub fn with_features(features: Features) -> Self {
        let dense_len = TokenSeq::from(token_vec![L L L]).as_usize() + 1;
        let mut table = PrefixTable::new(dense_len);
        for opcode in Opcode::iter() {
            if opcode.feature().map_or(true, |f| features.contains(f)) {
                let toks = Vec::from(opcode.tokens());
                table.register(&toks, opcode).unwrap();
            }
        }
        table
    }

    #[inline]
    pub fn with_all() -> Self {
        Self::with_features(Features::all())
    }
}

impl Debug for PrefixTable<Token> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        struct EntryDebug<'a>(TokenSeq<Token>, Option<&'a PrefixEntry>);
        impl<'a> Debug for EntryDebug<'a> {
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                write!(
                    f,
                    "{}{:?}: {:?}",
                    self.0.as_usize(),
                    TokenVec::from(self.0),
                    self.1
                )
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

        f.debug_struct("PrefixTable")
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

    #[test]
    fn optimal_size() {
        #[allow(dead_code)]
        enum PrefixEntryOf<T> {
            Terminal(Opcode),
            Prefix(T),
        }

        assert_eq_size!(
            PrefixEntry,
            PrefixEntryOf<Vec<Opcode>>,
            PrefixEntryOf<SmallVec<[Opcode; 16]>>,
            Option<PrefixEntry>,
            Option<PrefixEntryOf<Vec<Opcode>>>,
            Option<PrefixEntryOf<SmallVec<[Opcode; 16]>>>,
        );
        assert_eq_size!(SmallVec<[Opcode; 16]>, Vec<Opcode>, SmallVec<[Opcode; 16]>);
        const_assert!(size_of::<SmallVec<[Opcode; 16]>>() < size_of::<SmallVec<[Opcode; 17]>>());
    }
}
