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

use crate::syntax::{EnumIndex, TokenSeq};
use crate::text::EncodingError;
use crate::ws::inst::{Inst, InstArg, Opcode, RawInst};
use crate::ws::token::{Lexer, Token, Token::*};

#[derive(Clone, Debug)]
pub struct PrefixParser<'a, L: Lexer> {
    table: &'a PrefixTable<Token, Opcode>,
    lex: L,
    partial: Option<PartialState>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ParseError<T: Copy + EnumIndex, O> {
    EncodingError(EncodingError, TokenSeq<T>),
    UnknownOpcode(TokenSeq<T>),
    IncompleteInst(TokenSeq<T>, SmallVec<[O; 16]>),
    UnterminatedArg(Opcode, BitVec),
}

#[derive(Clone, Debug)]
enum PartialState {
    ParsingOpcode(TokenSeq<Token>),
    ParsingArg(Opcode, BitVec),
}

impl<'a, L: Lexer> PrefixParser<'a, L> {
    pub fn new(table: &'a PrefixTable<Token, Opcode>, lex: L) -> Self {
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
                        return Err(ParseError::EncodingError(err, tokens.into()));
                    }
                    None => return Err(ParseError::UnterminatedArg(opcode, bits)),
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
        // Restore state, if an instruction was interrupted with a lex error
        // after being partially parsed.
        let partial_seq = match self.partial.take() {
            Some(PartialState::ParsingOpcode(partial)) => partial,
            Some(PartialState::ParsingArg(opcode, bits)) => {
                return Some(self.parse_arg(opcode, Some(bits)));
            }
            None => TokenSeq::new(),
        };
        match self.table.parse_at(&mut self.lex, partial_seq)? {
            Ok(opcode) => return Some(self.parse_arg(opcode, None)),
            Err(err) => {
                if let ParseError::EncodingError(_, seq) = err {
                    self.partial = Some(PartialState::ParsingOpcode(seq));
                }
                return Some(Inst::from(ParseError::from(err)));
            }
        }
    }
}

impl<'a, L: Lexer + FusedIterator> const FusedIterator for PrefixParser<'a, L> {}

#[derive(Clone)]
pub struct PrefixTable<T, O> {
    dense: Box<[Option<PrefixEntry<O>>]>,
    sparse: HashMap<TokenSeq<T>, Option<PrefixEntry<O>>>,
}

#[derive(Clone, Debug)]
pub enum PrefixEntry<O> {
    Terminal(O),
    Prefix(SmallVec<[O; 16]>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TableError<T: Copy + EnumIndex, O> {
    Conflict(TokenSeq<T>, SmallVec<[O; 16]>),
    NoTokens(O),
}

impl<T, O> PrefixTable<T, O>
where
    T: Copy + EnumIndex + Eq,
    O: Copy,
{
    #[inline]
    pub fn new(dense_len: usize) -> Self {
        PrefixTable {
            dense: vec![None; dense_len].into_boxed_slice(),
            sparse: HashMap::new(),
        }
    }

    #[inline]
    pub fn with_dense_width(width: usize) -> Self {
        Self::new(TokenSeq::<T>::size_for(width))
    }

    #[inline]
    pub fn get(&self, seq: TokenSeq<T>) -> Option<&PrefixEntry<O>> {
        if seq.as_usize() < self.dense.len() {
            self.dense[seq.as_usize()].as_ref()
        } else {
            self.sparse.get(&seq).map(Option::as_ref).flatten()
        }
    }

    #[inline]
    pub fn get_mut(&mut self, seq: TokenSeq<T>) -> &mut Option<PrefixEntry<O>> {
        if seq.as_usize() < self.dense.len() {
            &mut self.dense[seq.as_usize()]
        } else {
            self.sparse.entry(seq).or_default()
        }
    }

    pub fn insert(&mut self, toks: &[T], opcode: O) -> Result<(), TableError<T, O>> {
        if toks.len() == 0 {
            return Err(TableError::NoTokens(opcode));
        }
        let mut seq = TokenSeq::new();
        for &tok in toks {
            let entry = self.get_mut(seq);
            match entry {
                Some(PrefixEntry::Terminal(terminal)) => {
                    return Err(TableError::Conflict(seq, smallvec![*terminal, opcode]));
                }
                Some(PrefixEntry::Prefix(opcodes)) => opcodes.push(opcode),
                None => *entry = Some(PrefixEntry::Prefix(smallvec![opcode])),
            }
            seq.push(tok);
        }
        let entry = self.get_mut(seq);
        match entry {
            Some(PrefixEntry::Terminal(terminal)) => {
                return Err(TableError::Conflict(seq, smallvec![*terminal, opcode]));
            }
            Some(PrefixEntry::Prefix(opcodes)) => {
                let mut opcodes = opcodes.clone();
                opcodes.push(opcode);
                return Err(TableError::Conflict(seq, opcodes));
            }
            None => *entry = Some(PrefixEntry::Terminal(opcode)),
        }
        Ok(())
    }

    pub fn parse<L>(&self, lex: &mut L) -> Option<Result<O, ParseError<T, O>>>
    where
        L: Iterator<Item = Result<T, EncodingError>>,
    {
        self.parse_at(lex, TokenSeq::new())
    }

    pub fn parse_at<L>(
        &self,
        lex: &mut L,
        partial: TokenSeq<T>,
    ) -> Option<Result<O, ParseError<T, O>>>
    where
        L: Iterator<Item = Result<T, EncodingError>>,
    {
        let mut seq = partial;
        loop {
            match lex.next() {
                Some(Ok(tok)) => seq.push(tok),
                Some(Err(err)) => return Some(Err(ParseError::EncodingError(err, seq))),
                None if seq.is_empty() => return None,
                None => {
                    let prefix = match self.get(seq) {
                        Some(PrefixEntry::Terminal(opcode)) => return Some(Ok(*opcode)),
                        Some(PrefixEntry::Prefix(opcodes)) => opcodes.clone(),
                        None => SmallVec::new(),
                    };
                    return Some(Err(ParseError::IncompleteInst(seq, prefix)));
                }
            }
            match self.get(seq) {
                Some(PrefixEntry::Terminal(opcode)) => return Some(Ok(*opcode)),
                Some(PrefixEntry::Prefix(_)) => {}
                None => return Some(Err(ParseError::UnknownOpcode(seq))),
            }
        }
    }
}

impl<T, O> Debug for PrefixTable<T, O>
where
    T: Copy + Debug + EnumIndex + Ord,
    O: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        struct EntryDebug<'a, T, O>(TokenSeq<T>, &'a Option<PrefixEntry<O>>);
        impl<'a, T: Copy + Debug + EnumIndex, O: Debug> Debug for EntryDebug<'a, T, O> {
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                write!(f, "{:?}: {:?}", self.0, self.1)
            }
        }

        let dense = self
            .dense
            .iter()
            .enumerate()
            .map(|(i, e)| EntryDebug(TokenSeq::<T>::from(i), e))
            .collect::<Vec<_>>();
        let mut sparse = self
            .sparse
            .iter()
            .map(|(&seq, e)| EntryDebug(seq, e))
            .collect::<Vec<_>>();
        sparse.sort_by(|a, b| a.0.cmp(&b.0));

        f.debug_struct("PrefixTable")
            .field("dense", &dense)
            .field("dense.len", &self.dense.len())
            .field("sparse", &sparse)
            .field("sparse.len", &self.sparse.len())
            .field("sparse.capacity", &self.sparse.capacity())
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
        enum PrefixEntryOf<T, P> {
            Terminal(T),
            Prefix(P),
        }

        assert_eq_size!(
            PrefixEntry<Opcode>,
            Option<PrefixEntry<Opcode>>,
            PrefixEntryOf<Opcode, Vec<Opcode>>,
            PrefixEntryOf<Opcode, SmallVec<[Opcode; 16]>>,
        );
        assert_eq_size!(Vec<Opcode>, SmallVec<[Opcode; 16]>);
        const_assert!(size_of::<SmallVec<[Opcode; 16]>>() < size_of::<SmallVec<[Opcode; 17]>>());
    }
}
